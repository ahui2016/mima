package main

import (
	"flag"
	"net/http"

	"ahui2016.github.com/mima/database"
	"ahui2016.github.com/mima/util"
)

const (
	OK              = http.StatusOK
	AppConfigFolder = "github-ahui2016/mima"
	dbFileName      = "db-mima.sqlite"
)

var (
	db   = new(database.DB)
	addr = flag.String("addr", "", "Local IP address. Example: 127.0.0.1:80")
)

func init() {
	flag.Parse()
	util.Panic(db.Open(dbFileName))
	if *addr == "" {
		s, err := db.GetSettings()
		util.Panic(err)
		*addr = s.AppAddr
	}
}
