package database

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"io/ioutil"

	"ahui2016.github.com/mima/util"
	"golang.org/x/crypto/nacl/secretbox"
)

const (
	KeySize   = 32
	NonceSize = 24
)

type (
	Nonce     = [NonceSize]byte
	SecretKey = [KeySize]byte
)

func newNonce() (nonce Nonce, err error) {
	_, err = rand.Read(nonce[:])
	return
}

func encrypt(data []byte, key SecretKey) ([]byte, error) {
	nonce, err := newNonce()
	if err != nil {
		return nil, err
	}
	sealed := secretbox.Seal(nonce[:], data, &nonce, &key)
	return sealed, nil
}

func decrypt(sealed []byte, key SecretKey) (mwh MimaWithHistory, err error) {
	var nonce Nonce
	copy(nonce[:], sealed[:NonceSize])
	mwhJSON, ok := secretbox.Open(nil, sealed[NonceSize:], &nonce, &key)
	if !ok {
		return mwh, errors.New("db.decrypt: secretbox open fail")
	}
	err = json.Unmarshal(mwhJSON, &mwh)
	return
}

func base64DecodeToSecretKey(s string) (*SecretKey, error) {
	b, err := util.Base64Decode(s)
	if err != nil {
		return nil, err
	}
	key := bytesToKey(b)
	return &key, nil
}

func bytesToKey(b []byte) (key SecretKey) {
	copy(key[:], b)
	return
}

func writeFile(fullPath, sealed64 string) error {
	return ioutil.WriteFile(fullPath, []byte(sealed64), 0644)
}

func RandomString64() string {
	size := 64
	someBytes := make([]byte, size)
	if _, err := rand.Read(someBytes); err != nil {
		panic(err)
	}
	return util.Base64Encode(someBytes)
}

func randomKey() SecretKey {
	return sha256.Sum256([]byte(RandomString64()))
}
