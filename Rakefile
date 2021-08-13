require 'securerandom'
require 'shellwords'

TARGET = ENV['TARGET'] || 'arm-unknown-linux-gnueabihf'

RPI = ENV['RPI'] || 'somfy-rts.local'
HOST = "pi@#{RPI}"

def ssh(*args)
  sh 'ssh', HOST, *args
end

desc 'compile binary'
task :build do
  sh 'cross', 'build', '--release', '--target', TARGET
end

desc 'set time zone on Raspberry Pi'
task :setup_timezone do
  ssh 'sudo', 'timedatectl', 'set-timezone', 'Europe/Vienna'
end

desc 'set hostname on Raspberry Pi'
task :setup_hostname do
  ssh <<~SH
    if ! dpkg -s dnsutils >/dev/null; then
      sudo apt-get update
      sudo apt-get install -y dnsutils
    fi

    hostname="$(dig -4 +short -x "$(hostname -I | awk '{print $1}')")"
    hostname="${hostname%%.local.}"

    if [ -n "${hostname}" ]; then
      echo "${hostname}" | sudo tee /etc/hostname >/dev/null
    fi
  SH
end

desc 'set up watchdog on Raspberry Pi'
task :setup_watchdog do
  ssh <<~SH
    if ! dpkg -s watchdog >/dev/null; then
      sudo apt-get update
      sudo apt-get install -y watchdog
    fi
  SH

  r, w = IO.pipe

  w.puts 'bcm2835_wdt'
  w.close

  ssh 'sudo', 'tee', '/etc/modules-load.d/bcm2835_wdt.conf', in: r

  gateway_ip = %x(#{['ssh', HOST, 'ip', 'route'].shelljoin})[/via (\d+.\d+.\d+.\d+) /, 1]

  raise if gateway_ip.empty?

  r, w = IO.pipe

  w.puts <<~CFG
    watchdog-device	= /dev/watchdog
    ping = #{gateway_ip}
  CFG
  w.close

  ssh 'sudo', 'tee', '/etc/watchdog.conf', in: r
  ssh 'sudo', 'systemctl', 'enable', 'watchdog'
end

task :setup => [:setup_timezone, :setup_hostname, :setup_watchdog]

task :install => :build do
  sh 'rsync', '-z', '--rsync-path', 'sudo rsync', "target/#{TARGET}/release/somfy", "#{HOST}:/usr/local/bin/somfy"
end

task :deploy => :install
