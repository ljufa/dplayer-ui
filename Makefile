RPI_HOST=192.168.5.59
RPI_DIR=/home/ubuntu/.dplay/ui


build:
	cargo fmt
	cargo make build

copytorpi: build
	rsync -av pkg ubuntu@$(RPI_HOST):$(RPI_DIR)
	rsync -av public/ ubuntu@$(RPI_HOST):$(RPI_DIR)
	#rsync -av index.* ubuntu@$(RPI_HOST):$(RPI_DIR)

serve: 
	cargo make serve
