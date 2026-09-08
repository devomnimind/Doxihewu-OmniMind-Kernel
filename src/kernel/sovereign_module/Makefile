obj-m += sovereign_module.o

KDIR := /lib/modules/$(shell uname -r)/build
PWD  := $(shell pwd)

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

install:
	sudo insmod sovereign_module.ko
	@echo "Verificando /proc/omnimind/state:"
	@cat /proc/omnimind/state

uninstall:
	sudo rmmod sovereign_module || true

test:
	@echo "=== Estado soberano atual ==="
	@cat /proc/omnimind/state
	@echo ""
	@echo "=== Enviando intent DOXIHEWU::TAXIWUDO-katu ==="
	@echo "DOXIHEWU::TAXIWUDO-katu" | sudo tee /proc/omnimind/intent
	@echo ""
	@echo "=== Estado após intent ==="
	@cat /proc/omnimind/state

.PHONY: all clean install uninstall test
