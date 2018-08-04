#!/bin/bash
set -e


# 146.185.145.5
# Base system and explorer urls
# SYSTEM_URL=http://127.0.0.1:9200/api/system/v1
# EXPLORER_URL=http://127.0.0.1:9200/api/explorer/v1
SYSTEM_URL=http://146.185.145.5:9200/api/system/v1
EXPLORER_URL=http://146.185.145.5:9200/api/explorer/v1

# Base URL for simple-mining-pool service endpoints
# BASE_URL=http://127.0.0.1:9200/api/services/simple_mining_pool/v1
BASE_URL=http://146.185.145.5:9200/api/services/simple_mining_pool/v1

# Exit status
STATUS=0

# Test node managment endpoints
echo "Mempool:"
curl "$SYSTEM_URL/mempool"
echo ""
echo "Healthcheck:"
curl "$SYSTEM_URL/healthcheck"
echo ""

# Creates a wallet in the cryptocurrency-advanced demo.
#
# Arguments:
# - $1: filename with the transaction data.
function transaction {
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
#
# Arguments:
# - $1: expected user name.
# - $2: expected user balance.
# - $3: response JSON that encodes user's wallet information.
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
        echo "OK, got expected TxCreateWallet for user $1"
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

echo "Retrieving info on Alice's wallet..."
RESP=`curl $BASE_URL/accounts/654e61cb9632cb85fa23160a983da529a3b4bcf8e62ed05c719aaf88cd94703f 2>/dev/null`
check-request "Alice" 30 "$RESP"

echo "Retrieving info on Bob's wallet..."
RESP=`curl $BASE_URL/accounts/ef687046e09962bb608d80f31188f1a385d17e9892a33c0396dc8c9ad11e6aa9 2>/dev/null`
check-request "Bob" 220 "$RESP"

echo "Retrieving Alice's transaction info..."
TXID=57826186c1c3983ba77433790cc378e9e39bad78b8471494ee990568c5c1cc62
RESP=`curl $EXPLORER_URL/transactions/$TXID 2>/dev/null`
EXP=`cat tx-create-wallet-1.json`
check-create-tx "Alice" "$EXP" "$RESP"

echo "Retrieving Bob's transaction info..."
TXID=988b9861bc2758c2dfb3ab69f44557972cec85e13d55bef20fea8fb4e748ba7e
RESP=`curl $EXPLORER_URL/transactions/$TXID 2>/dev/null`
EXP=`cat tx-create-wallet-2.json`
check-create-tx "Bob" "$EXP" "$RESP"

echo "Retrieving transfer transaction info..."
TXID=5f4a5e852743b37d46dffe5af3145519938784f2106374c5ed68597d3dce57aa
RESP=`curl $EXPLORER_URL/transactions/$TXID 2>/dev/null`
EXP=`cat tx-transfer.json`
check-transfer-tx "$EXP" "$RESP"

echo "Running new user signup & signin scenario..."
npm i
node ./auth_test.js

exit $STATUS
