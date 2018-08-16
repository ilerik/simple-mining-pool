#!/bin/bash

#
# Bash script for testing the simple mining service instance
#

set -e # Exit immediately

# Host parameters
HOST_IP=${1:-127.0.0.1}
HOST_PUBLIC_API_PORT=8200
HOST_PRIVATE_API_PORT=8091

# accepted discrepancy between system time and time returned by service
# seconds
ACCEPTED_TIME_VARIANCE=10

# global variables used to build url.
# using global variables is not a good practice, but I can't find better alternative to organize url builder.
url_builder_host_ip=$HOST_IP
url_builder_port=$HOST_PUBLIC_API_PORT
url_builder_service="simple_mining_pool"
# possible values: system, explorer or service
url_builder_api_type="services"

function with-private-api {
    verify-number-of-input-params "$#" 0
    url_builder_port=$HOST_PRIVATE_API_PORT
}

function with-service {
#   exactly one parameter is expected
    verify-number-of-input-params "$#" 1
    url_builder_service=$1
}

function with-api-type {
#   exactly one parameter is expected
    verify-number-of-input-params "$#" 1
    url_builder_api_type=$1
}

# idea is that most often default values are used
# and after using build-url function with custom parameters
# it is possible to build default url without calling any with-... functions
function setup-default-url-builder-values {
    verify-number-of-input-params "$#" 0
    url_builder_host_ip=$HOST_IP
    url_builder_port=$HOST_PUBLIC_API_PORT
    url_builder_service="simple_mining_pool"
    # possible values: system, explorer or service
    url_builder_api_type="services"
}

# function builds url from global vars and returns it
#
# No arguments:
function build-url {
    verify-number-of-input-params "$#" 0
    if [ "$url_builder_api_type" == "services" ]; then
        BASE_URL=http://$url_builder_host_ip:$url_builder_port/api/$url_builder_api_type/$url_builder_service/v1
    else
        BASE_URL=http://$url_builder_host_ip:$url_builder_port/api/$url_builder_api_type/v1
    fi
    setup-default-url-builder-values
}

# idea: reuse functionality to check number of input parameters to function
# function verifies that second
#
# Argiments:
#  - $1: actual number of parameters
#  - $2: expected number of parameters
function verify-number-of-input-params {
        if [ "$1" -ne "$2" ]; then
            echo "exactly $2 parameter(s) is/are expected"
            echo "$1 parameter(s) is/are supplied"
            STATUS=1
        fi
}

# function asks service for current_time, prints and verifies it
function verify-service-current-time {
    setup-default-url-builder-values
    with-service "exonum_time"
    build-url
    RESP=`curl $BASE_URL/current_time 2>/dev/null`
    echo "service current_time: $RESP"
    #   format of time according to https://exonum.com/doc/advanced/time/#current-time
    verify_time $RESP
}

# function verifies that input date differs no more than ACCEPTED_TIME_VARIANCE seconds from system time
# if input differs more than ACCEPTED_TIME_VARIANCE seconds from system time, than
# Exit status
# STATUS
# is changed to 1
#
# Arguments:
# - $1: date in ico8601 format
# it appears that parsing json gives string with quotes, so quotes from start and end of the input string are removed
function verify_time {
#   remove quotes
#   code from https://stackoverflow.com/a/26314887
    local input_utc_ico8601=$(echo "$1" | tr -d '"')
    local input_seconds=`date -d$input_utc_ico8601 +%s`
    local system_time_seconds=$(date -u +%s)
    local dif=$((system_time_seconds-input_seconds))
#   for abs function code is taken from https://stackoverflow.com/a/47240327
    local abs_dif=${dif#-}
    if [ "$abs_dif" -gt "$ACCEPTED_TIME_VARIANCE" ]; then
        echo "Test Error: system time $(date) differs from input time $1 by more than $ACCEPTED_TIME_VARIANCE seconds"
        STATUS=1
    else
        echo "system time $(date) differs from input time $1 by $abs_dif seconds"
    fi
}

# function asks service for validators_times/all, prints and verifies them
function verify-validators-times {
    setup-default-url-builder-values
    with-service "exonum_time"
    with-private-api
    build-url
    RESP=`curl $BASE_URL/validators_times/all 2>/dev/null`
    local i=0
#   loop through json according to code from https://starkandwayne.com/blog/bash-for-loop-over-json-array-using-jq/
    for row in $(echo "${RESP}" | jq -r '.[] | @base64'); do
        _jq() {
            echo ${row} | base64 --decode | jq -r ${1}
        }

        echo validator_${i}_public_key: $(_jq '.public_key')
        echo validator_${i}_time: $(_jq '.time')
        verify_time $(_jq '.time')
        i+=1
    done
}


# Exit status
STATUS=0

# Test node managment endpoints
setup-default-url-builder-values
with-api-type "system"
build-url
echo "Mempool:"
curl "$BASE_URL/mempool"
echo ""
echo "Healthcheck:"
curl "$BASE_URL/healthcheck"
echo ""

# Runs docker container.
function launch-server {
    docker run -d -p 8200:8200 $BASE_IMAGE & sleep 10
}

function kill-server {
    docker ps | grep $BASE_IMAGE | gawk '{print $1}' | xargs docker stop || true
}

# Creates a wallet in the cryptocurrency-advanced demo.
#
# Arguments:
# - $1: filename with the transaction data.
function transaction {
    setup-default-url-builder-values
    build-url
    RESP=`curl -H "Content-Type: application/json" -X POST -d @$1 $BASE_URL/transaction 2>/dev/null`
    sleep 1
}

# Checks a response to an Exonum transaction.
#
# Arguments:
# - $1: expected start of the transaction hash returned by the server.
function check-transaction {
    if [[ `echo $RESP | jq .tx_hash` =~ ^\"$1 ]]; then
        echo "OK, got expected transaction hash $1"
    else
        echo "Unexpected response: $RESP"
        STATUS=1
    fi
}

# Checks a response to a read request.

function check-request {
    if [[ ( `echo $3 | jq .name` == "\"$1\"" ) && ( `echo $3 | jq .balance` == "\"$2\"" ) ]]; then
        echo "OK, got expected transaction balance $2 for user $1"
    else
        # $RESP here is intentional; we want to output the entire incorrect response
        echo "Unexpected response: $RESP"
        STATUS=1
    fi
}

# Checks a `CreateWallet` transaction in the blockchain explorer.
#
# Arguments:
# - $1: expected user name
# - $2: expected transaction JSON
# - $3: response JSON
function check-create-tx {
    if [[ \
      ( `echo $3 | jq .type` == \"committed\" ) && \
      ( `echo $3 | jq .content.body.name` == "\"$1\"" ) && \
      ( `echo $3 | jq ".content == $2"` == "true" ) \
    ]]; then
        echo "OK, got expected TxCreateAccount for user $1"
    else
        echo "Unexpected response: $3"
        STATUS=1
    fi
}

# Checks a `Transfer` transaction in the blockchain explorer.
#
# Arguments:
# - $1: expected transaction JSON
# - $2: response JSON
function check-transfer-tx {
    if [[ \
      ( `echo $2 | jq .type` == \"committed\" ) && \
      ( `echo $2 | jq ".content == $1"` == "true" ) \
    ]]; then
        echo "OK, got expected TxTransfer between wallets"
    else
        echo "Unexpected response: $2"
        STATUS=1
    fi
}

#launch-server
verify-service-current-time
verify-validators-times

echo "Creating a account for Alice..."
transaction tx-create-wallet-1.json
check-transaction 57826186

echo "Creating a account for Bob..."
transaction tx-create-wallet-2.json
check-transaction 988b9861

echo "Add funds to Alice's account..."
transaction tx-issue.json
check-transaction 8aa865f9

echo "Transferring funds from Alice to Bob..."
transaction tx-transfer.json
check-transaction 5f4a5e85

echo "Waiting until transactions are committed..."
sleep 5
verify-service-current-time
verify-validators-times

echo "Retrieving info on Alice's wallet..."
setup-default-url-builder-values
with-service "simple_mining_pool"
build-url
RESP=`curl $BASE_URL/accounts?pub_key=654e61cb9632cb85fa23160a983da529a3b4bcf8e62ed05c719aaf88cd94703f 2>/dev/null`
check-request "Alice" 30 "$RESP"

echo "Retrieving info on Bob's wallet..."
setup-default-url-builder-values
with-service "simple_mining_pool"
build-url
RESP=`curl $BASE_URL/accounts?pub_key=ef687046e09962bb608d80f31188f1a385d17e9892a33c0396dc8c9ad11e6aa9 2>/dev/null`
check-request "Bob" 220 "$RESP"

echo "Retrieving Alice's transaction info..."
setup-default-url-builder-values
with-api-type "explorer"
build-url
TXID=57826186c1c3983ba77433790cc378e9e39bad78b8471494ee990568c5c1cc62
RESP=`curl $BASE_URL/transactions?hash=$TXID 2>/dev/null`
EXP=`cat tx-create-wallet-1.json`
check-create-tx "Alice" "$EXP" "$RESP"

echo "Retrieving Bob's transaction info..."
setup-default-url-builder-values
with-api-type "explorer"
build-url
TXID=988b9861bc2758c2dfb3ab69f44557972cec85e13d55bef20fea8fb4e748ba7e
RESP=`curl $BASE_URL/transactions?hash=$TXID 2>/dev/null`
EXP=`cat tx-create-wallet-2.json`
check-create-tx "Bob" "$EXP" "$RESP"

echo "Retrieving transfer transaction info..."
setup-default-url-builder-values
with-api-type "explorer"
build-url
TXID=5f4a5e852743b37d46dffe5af3145519938784f2106374c5ed68597d3dce57aa
RESP=`curl $BASE_URL/transactions?hash=$TXID 2>/dev/null`
EXP=`cat tx-transfer.json`
check-transfer-tx "$EXP" "$RESP"
verify-service-current-time
verify-validators-times

#kill-server

exit $STATUS
