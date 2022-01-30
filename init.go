package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"ahui2016.github.com/mima/database"
	"ahui2016.github.com/mima/util"
)

const (
	AppConfigFolder = "github-ahui2016/mima"
	dbFileName      = "db-mima.sqlite"
)

var (
	db   = new(database.DB)
	addr = flag.String("addr", "", "Local IP address. Example: 127.0.0.1:80")
)

func init() {
	userConfigDir, err := os.UserConfigDir()
	util.Panic(err)
	configFolder := filepath.Join(userConfigDir, AppConfigFolder)
	os.MkdirAll(configFolder, 0640)
	dbPath := filepath.Join(configFolder, dbFileName)
	fmt.Println("[Database]", dbPath)

	flag.Parse()
	util.Panic(db.Open(dbPath))
	if *addr == "" {
		s, err := db.GetSettings()
		util.Panic(err)
		*addr = s.AppAddr
	}
}
