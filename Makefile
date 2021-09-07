RPI_HOST=192.168.44.26
RPI_DIR=/home/ubuntu/.dplay/ui


build:
	cargo fmt
	cargo make build

copytorpi: build
	rsync -av pkg ubuntu@$(RPI_HOST):$(RPI_DIR)
	rsync -av public/ ubuntu@$(RPI_HOST):$(RPI_DIR)
	# rsync -av index.* ubuntu@$(RPI_HOST):$(RPI_DIR)

serve: 
	cargo make serve
