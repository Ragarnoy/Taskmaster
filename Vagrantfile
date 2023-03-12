Vagrant.configure("2") do |config|
    config.vm.box = "ubuntu/focal64"
    config.vm.hostname = "Taskmaster"

    config.vm.provider "virtualbox" do |vb|
        vb.name = "Taskmaster-vm"
        vb.memory = "1024"
        vb.cpus = 2
    end
    config.vm.provision :shell, inline: "sudo apt update -y"
    config.vm.provision :shell, inline: "sudo apt install -y clang"
    config.vm.provision :shell, inline: "echo 'curl https://sh.rustup.rs/ -sSf | sh -s -- -y;' | su vagrant"
end
