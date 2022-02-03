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
	minute         = 60
	defaultMaxAge  = 30 * minute
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

// checkPasswordAndIP 检查 IP 与密码，返回 true 表示需要结束函数。
func checkPasswordAndIP(c *gin.Context, pwd string) (exit bool) {
	ip := c.ClientIP()
	if err := checkIPTryCount(ip); err != nil {
		c.JSON(http.StatusTooManyRequests, Text{err.Error()})
		return true
	}
	yes, err := db.CheckPassword(pwd)
	util.Panic(err)
	if !yes {
		ipTryCount[ip]++
		c.JSON(http.StatusUnauthorized, Text{"wrong password"})
		return true
	}
	ipTryCount[ip] = 0
	return false
}

func isSignedIn(c *gin.Context) bool {
	session := sessions.Default(c)
	yes, _ := session.Get(cookieSignIn).(bool)
	return yes
}

func CheckSignIn() gin.HandlerFunc {
	return func(c *gin.Context) {
		if !isSignedIn(c) {
			c.AbortWithStatusJSON(http.StatusUnauthorized, Text{"require sign-in"})
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

func newNormalOptions() sessions.Options {
	return newOptions(defaultMaxAge)
}

func newExpireOptions() sessions.Options {
	return newOptions(-1)
}

func newOptions(maxAge int) sessions.Options {
	return sessions.Options{
		Path:     "/",
		MaxAge:   maxAge,
		Secure:   true,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	}
}

func sessionSet(s sessions.Session, val bool, options sessions.Options) error {
	s.Set(cookieSignIn, val)
	s.Options(options)
	return s.Save()
}
