package main

import (
	"crypto/rand"
	"fmt"
	"net/http"

	"ahui2016.github.com/mima/util"
	"github.com/gin-contrib/sessions"
	"github.com/gin-gonic/gin"
)

const (
	sessionName    = "mima-session"
	cookieSignIn   = "mima-cookie-signin"
	passwordMaxTry = 5
)

var ipTryCount = make(map[string]int)

func checkIPTryCount(ip string) error {
	if *demo {
		return nil // 演示版允许无限重试密码
	}
	if ipTryCount[ip] >= passwordMaxTry {
		return fmt.Errorf("no more try, input wrong password too many times")
	}
	return nil
}

func isSignedIn(c *gin.Context) bool {
	session := sessions.Default(c)
	yes, _ := session.Get(cookieSignIn).(bool)
	return yes
}

func CheckSignIn() gin.HandlerFunc {
	return func(c *gin.Context) {
		if !isSignedIn(c) {
			c.Status(http.StatusUnauthorized)
			return
		}
		c.Next()
	}
}

func generateRandomKey() []byte {
	b := make([]byte, 32)
	_, err := rand.Read(b)
	util.Panic(err)
	return b
}
