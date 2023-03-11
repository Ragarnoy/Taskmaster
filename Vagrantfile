
Vagrant.configure("2") do |config|
    config.vm.box = "ubuntu/focal64"
    config.vm.hostname = "Taskmaster"

    config.vm.provider "virtualbox" do |vb|
        vb.name = "Taskmaster-vm"
        vb.memory = "1024"
        vb.cpus = 2
    end
