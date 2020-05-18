API_TAG ?= latest
API_IMAGE = gcr.io/aspen-lang/platform/api
API_BUILDER_IMAGE = gcr.io/aspen-lang/platform/api-builder

.PHONY: build
build:
	docker build \
		--target builder \
		--cache-from $(API_BUILDER_IMAGE):latest \
		--tag $(API_BUILDER_IMAGE):$(API_TAG) \
		api
	docker build \
		--cache-from $(API_BUILDER_IMAGE):$(API_TAG) \
		--cache-from $(API_IMAGE):latest \
		--tag $(API_IMAGE):$(API_TAG) \
		api

.PHONY: push
push: build
	docker push $(API_IMAGE):$(API_TAG)
	gcloud run deploy aspen-api \
		--image=$(API_IMAGE):$(API_TAG) \
		--region europe-north1 \
		--platform managed
	docker push $(API_BUILDER_IMAGE):$(API_TAG)
	docker tag $(API_BUILDER_IMAGE):$(API_TAG) $(API_BUILDER_IMAGE):latest
	docker push $(API_BUILDER_IMAGE):latest
	docker tag $(API_IMAGE):$(API_TAG) $(API_IMAGE):latest
	docker push $(API_IMAGE):latest

.PHONY: pull
pull:
	docker pull $(API_BUILDER_IMAGE):latest
	docker pull $(API_IMAGE):latest
