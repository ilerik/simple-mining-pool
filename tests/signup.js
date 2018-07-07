"use strict";

const bip39 = require('bip39');
const crypto = require('crypto');
const secp256k1 = require('secp256k1');
const bitcoin = require('bitcoinjs-lib');
const ethUtils = require('ethereumjs-util');
const Exonum = require('exonum-client');

const fetch = require('isomorphic-fetch');

function createAccount() {
    let privKey = '0a8779e7a5edeaf71455a06205493fd5c4b4623d24755edc4013238145cd7982814bca90d29c116b62e6d97a11a7178ac43920b6169654a79ed457a863b0f53e';
    let pubKey = '814bca90d29c116b62e6d97a11a7178ac43920b6169654a79ed457a863b0f53e';

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
    return TxCreateAccount.send('http://127.0.0.1:9200/api/services/simple_mining_pool/v1/transaction', 'http://127.0.0.1:9200/api/explorer/v1/transactions?hash=', data, signature)
    .then(() => {
      return { data: { tx_hash : hash } }
    });
}

// Create account for new user

const data = createAccount();
// expect(data.data).toEqual({
// 'tx_hash': '8055cd33cf11106f16321feb37777c3a92cbeaa23b9f7984a5b819ae51fee596'
// })

return;

function checkStatus(response) {
    if (response.status >= 200 && response.status < 300) {
        return response
    } else {
        var error = new Error(response.statusText)
        error.response = response
        throw error
    }
}

function parseJSON(response) {
    return response.json()
}

function verbose_fetch(url, request) {
    fetch(url, request)
    .then(checkStatus)
    .then(function(response) {
        console.log("Content-Type" + response.headers.get('Content-Type'))
        console.log("Date" + response.headers.get('Date'))
        console.log("Status" + response.status)
        console.log("Status text" + response.statusText)
        return response
    })
    .catch(function(error) {
      console.log('request failed', error)
    })    
}

// Signup new user
let mnemonic = bip39.entropyToMnemonic('00000000000000000000000000000000');
let key = bip39.mnemonicToEntropy(mnemonic);
let pubKey = secp256k1.publicKeyCreate(key)
console.log("Test mnemonic:", phrase);
console.log("Mnemonic validity test:", bip39.validateMnemonic(mnemonic));
console.log("Public key:", pubKey);

// Derive BTC address
let seedBuffer = bip39.mnemonicToSeed(phrase);
let masterNode = bitcoin.HDNode.fromSeedBuffer(seedBuffer)
let account0 = masterNode.derivePath("m/44'/0'/0'")
let xpubString = account0.neutered().toBase58();

let key0 = account0.derivePath("0/0").keyPair
let key0FromXpub = account0.neutered().derivePath("0/0").keyPair

let address0 = key0.getAddress()
let address0FromXpub = key0FromXpub.getAddress();
let address0FromXpubKey = bitcoin.HDNode.fromBase58(xpubString);

// ETH address
var ethAddress = ethUtils.privateToAddress(this.state.key).toString('hex');
console.log("ETH public address:", ethAddress);

var username = "ilerik";
var uid = pubKey;
var proof = "";
var payload = JSON.stringify({ "uid": uid, "username":username, "proof": proof });
// verbose_fetch("http://127.0.0.1:3000/signup", {
//     method: 'post',  
//     headers: {        
//         'Accept': 'application/json',
//         'Content-Type': 'text/javascript'
//     },
//     body: payload,
// })

