#!/usr/bin/expect -f

set program [lindex $argv 0]

if { $program == "mysql" } {
    spawn dpkg -i mysql-apt-config_0.8.25-1_all.deb
    expect "Which MySQL product do you wish to configure?"
    send "1\r"
    expect "Which server version do you wish to receive?"
    send "1\r"
    expect "Which MySQL product do you wish to configure?"
    send "4\r"
	expect eof
} else {
	spawn apt install -y mysql-server
	expect "Enter root password:"
    send "\r"
	expect "Select default authentication plugin"
    send "2\r"
	expect eof
}
