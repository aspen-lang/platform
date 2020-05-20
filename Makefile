CACHE_TAG ?= latest
API_TAG ?= latest
API_IMAGE = gcr.io/aspen-lang/platform/api
API_BUILDER_IMAGE = gcr.io/aspen-lang/platform/api-builder
API_SERVICE_NAME ?= aspen-api

.PHONY: build
build:
	docker build \
		--target builder \
		--cache-from $(API_BUILDER_IMAGE):$(CACHE_TAG) \
		--tag $(API_BUILDER_IMAGE):$(API_TAG) \
		api
	docker build \
		--cache-from $(API_BUILDER_IMAGE):$(API_TAG) \
		--cache-from $(API_IMAGE):$(CACHE_TAG) \
		--tag $(API_IMAGE):$(API_TAG) \
		api

.PHONY: push
push: build
	docker push $(API_IMAGE):$(API_TAG)
	gcloud run deploy $(API_SERVICE_NAME) \
		--image=$(API_IMAGE):$(API_TAG) \
		--region europe-north1 \
		--platform managed
	docker push $(API_BUILDER_IMAGE):$(API_TAG)
	docker tag $(API_BUILDER_IMAGE):$(API_TAG) $(API_BUILDER_IMAGE):$(CACHE_TAG)
	docker push $(API_BUILDER_IMAGE):$(CACHE_TAG)
	docker tag $(API_IMAGE):$(API_TAG) $(API_IMAGE):$(CACHE_TAG)
	docker push $(API_IMAGE):$(CACHE_TAG)

.PHONY: pull
pull:
	docker pull $(API_BUILDER_IMAGE):$(CACHE_TAG) || true
	docker pull $(API_IMAGE):$(CACHE_TAG) || true
