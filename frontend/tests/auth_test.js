"use strict";

const bip39 = require('bip39');
const crypto = require('crypto');
const secp256k1 = require('secp256k1');
const bitcoin = require('bitcoinjs-lib');
const ethUtils = require('ethereumjs-util');
const Exonum = require('exonum-client');

const fetch = require('isomorphic-fetch');

// Constants
let privKey = '0a8779e7a5edeaf71455a06205493fd5c4b4623d24755edc4013238145cd7982814bca90d29c116b62e6d97a11a7178ac43920b6169654a79ed457a863b0f53e';
let pubKey = '814bca90d29c116b62e6d97a11a7178ac43920b6169654a79ed457a863b0f53e';

function createAccount() {    

    const CreateTransaction = {
      protocol_version: 0,
      service_id: 128,
      message_id: 2,
      fields: [
        { name: 'pub_key', type: Exonum.PublicKey },
        { name: 'name', type: Exonum.String }
      ]
    }

    const TxCreateAccount = Exonum.newMessage(CreateTransaction);

    const data = {
      pub_key: pubKey,
      name: 'John Doe'
    }

    const signature = TxCreateAccount.sign(privKey, data)

    TxCreateAccount.signature = signature

    const hash = TxCreateAccount.hash(data);
    console.log(TxCreateAccount)
    return TxCreateAccount.send('http://127.0.0.1:9200/api/services/simple_mining_pool/v1/transaction', 
        'http://127.0.0.1:9200/api/explorer/v1/transactions/', data, signature)
    .then(() => {
      return { data: { tx_hash : hash } }
    });
}

// Sign in user created
function signIn() {

    const CreateTransaction = {
        protocol_version: 0,
        service_id: 128,
        message_id: 3,
        fields: [
          { name: 'pub_key', type: Exonum.PublicKey },
          { name: 'name', type: Exonum.String }
        ]
      }
    
    const TxSignIn = Exonum.newMessage(CreateTransaction);

    const data = {
      pub_key: pubKey,
      name: 'John Doe'
    }

    const signature = TxSignIn.sign(privKey, data)

    TxSignIn.signature = signature

    const hash = TxSignIn.hash(data);
    console.log(TxSignIn)
    return TxSignIn.send('http://127.0.0.1:9200/api/services/simple_mining_pool/v1/transaction', 
        'http://127.0.0.1:9200/api/explorer/v1/transactions/', data, signature)
    .then(() => {
      return { data: { tx_hash : hash } }
    });
}

function checkStatus(response) {
    if (response.status >= 200 && response.status < 300) {
        return response
    } else {
        var error = new Error(response.statusText)
        error.response = response
        throw error
    }
}

//
async function main() {
    const data = await createAccount();
    const jwt = await signIn();
    return data;
}

main();
