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

var trustedIPs = make(map[string]bool)
var PIN = "1234" // 简短密码，被记录在 trustedIPs 里的 IP 可用。

func checkIPTryCount(ip string) error {
	if *demo {
		return nil // 演示版允许无限重试密码
	}
	if ipTryCount[ip] >= passwordMaxTry {
		return fmt.Errorf("no more try, input wrong password too many times")
	}
	return nil
}

// checkPasswordAndIP 检查 IP 与密码，返回 true 表示有错误。
func checkPasswordAndIP(c *gin.Context, pwd string) (exit bool) {
	ip := c.ClientIP()

	// 先检查是否超过尝试次数
	if err := checkIPTryCount(ip); err != nil {
		c.JSON(http.StatusForbidden, Text{err.Error()})
		return true
	}

	// 再验证 PIN, 如果正确就不需要检查主密码。
	if trustedIPs[ip] && pwd == PIN {
		ipTryCount[ip] = 0
		return false
	}

	// 如果 PIN 不正确，再验证主密码。
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

// checkPinAndIP 检查 IP 与 PIN 码，返回 true 表示有错误。
func checkPinAndIP(c *gin.Context, pin string) (exit bool) {
	ip := c.ClientIP()
	if err := checkIPTryCount(ip); err != nil {
		c.JSON(http.StatusForbidden, Text{err.Error()})
		return true
	}
	if pin != PIN {
		ipTryCount[ip]++
		c.JSON(http.StatusUnauthorized, Text{"PIN码错误（注意，每次后端重启都会恢复默认PIN码）"})
		return true
	}
	ipTryCount[ip] -= 1
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
