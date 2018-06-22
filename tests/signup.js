"use strict";

const bip39 = require('bip39');
const crypto = require('crypto');
const secp256k1 = require('secp256k1');
const bitcoin = require('bitcoinjs-lib');
const ethUtils = require('ethereumjs-util');

const fetch = require('isomorphic-fetch');

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
verbose_fetch("http://127.0.0.1:3000/signup", {
    method: 'post',  
    headers: {        
        'Accept': 'application/json',
        'Content-Type': 'text/javascript'
    },
    body: payload,
})