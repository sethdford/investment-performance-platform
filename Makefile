.PHONY: build test clean deploy-dev deploy-test deploy-prod

# Variables
ENVIRONMENT ?= dev
REGION ?= us-east-1
LAMBDA_BUCKET ?= investment-performance-lambda-code-$(ENVIRONMENT)
STACK_NAME ?= investment-performance-$(ENVIRONMENT)

# Build targets
build: build-api-handler build-event-processor build-performance-calculator

build-api-handler:
	@echo "Building API Handler..."
	cd api-handler && cargo build --release
	mkdir -p dist
	cp target/release/api-handler dist/bootstrap
	cd dist && zip -r ../api-handler.zip bootstrap

build-event-processor:
	@echo "Building Event Processor..."
	cd event-processor && cargo build --release
	mkdir -p dist
	cp target/release/event-processor dist/bootstrap
	cd dist && zip -r ../event-processor.zip bootstrap

build-performance-calculator:
	@echo "Building Performance Calculator..."
	cd performance-calculator && cargo build --release
	mkdir -p dist
	cp target/release/performance-calculator dist/bootstrap
	cd dist && zip -r ../performance-calculator.zip bootstrap

# Test targets
test:
	@echo "Running tests..."
	cargo test

test-integration:
	@echo "Running integration tests..."
	cargo test --test integration_tests

# Clean targets
clean:
	@echo "Cleaning..."
	cargo clean
	rm -rf dist
	rm -f *.zip

# Deploy targets
create-lambda-bucket:
	@echo "Creating Lambda code bucket..."
	aws s3api create-bucket --bucket $(LAMBDA_BUCKET) --region $(REGION)

upload-lambda-code: build
	@echo "Uploading Lambda code to S3..."
	aws s3 cp api-handler.zip s3://$(LAMBDA_BUCKET)/api-handler.zip
	aws s3 cp event-processor.zip s3://$(LAMBDA_BUCKET)/event-processor.zip
	aws s3 cp performance-calculator.zip s3://$(LAMBDA_BUCKET)/performance-calculator.zip

deploy-cloudformation: upload-lambda-code
	@echo "Deploying CloudFormation stack..."
	aws cloudformation deploy \
		--template-file infrastructure/cloudformation.yaml \
		--stack-name $(STACK_NAME) \
		--parameter-overrides \
			Environment=$(ENVIRONMENT) \
			ApiHandlerFunctionS3Key=api-handler.zip \
			EventProcessorFunctionS3Key=event-processor.zip \
			PerformanceCalculatorFunctionS3Key=performance-calculator.zip \
			LambdaCodeS3Bucket=$(LAMBDA_BUCKET) \
		--capabilities CAPABILITY_IAM CAPABILITY_NAMED_IAM \
		--region $(REGION)

deploy-waf: 
	@echo "Deploying WAF..."
	aws cloudformation deploy \
		--template-file infrastructure/waf.yaml \
		--stack-name $(STACK_NAME)-waf \
		--parameter-overrides \
			Environment=$(ENVIRONMENT) \
			ApiGatewayId=$$(aws apigateway get-rest-apis --query "items[?name=='investment-performance-api-$(ENVIRONMENT)'].id" --output text) \
		--capabilities CAPABILITY_IAM \
		--region $(REGION)

deploy-dashboard:
	@echo "Deploying CloudWatch dashboard..."
	aws cloudformation deploy \
		--template-file infrastructure/cloudwatch-dashboard.yaml \
		--stack-name $(STACK_NAME)-dashboard \
		--parameter-overrides \
			Environment=$(ENVIRONMENT) \
		--region $(REGION)

deploy-backup:
	@echo "Deploying backup configuration..."
	aws cloudformation deploy \
		--template-file infrastructure/backup.yaml \
		--stack-name $(STACK_NAME)-backup \
		--parameter-overrides \
			Environment=$(ENVIRONMENT) \
			DynamoDBTableName=investment-performance-$(ENVIRONMENT) \
		--capabilities CAPABILITY_IAM \
		--region $(REGION)

deploy-all: deploy-cloudformation deploy-waf deploy-dashboard deploy-backup

deploy-dev: ENVIRONMENT=dev
deploy-dev: deploy-all

deploy-test: ENVIRONMENT=test
deploy-test: deploy-all

deploy-prod: ENVIRONMENT=prod
deploy-prod: deploy-all 