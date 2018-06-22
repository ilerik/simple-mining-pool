# simple-mining-pool
Simple mining pool project

## Pool
To start the pool server:

```
cargo run --target-dir pool
```
RESTful API should be availible at http://localhost:3000/ and implement following endpoints:

SignUp (registering new user workflow):
1. POST to /signup: json!{ uid, credentials, proof }

SignIn (authorization workflow):
1. POST to /signin: json!{ credentials, proof }
1. Get response containing JWT for further authentification

Where proof is cryptographic signature created using user owned private keys.

## Tests
To run tests from /tests folder:

```
npm install
npm run test
```

## Client

Client library written in rust and compiled to pure wasm target resides inside /client folder. WIP