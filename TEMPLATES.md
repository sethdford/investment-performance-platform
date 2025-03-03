# AWS SAM Templates

This project includes two AWS SAM templates for different deployment scenarios:

## Production Template (`template-production.yaml`)

The production template provides a comprehensive infrastructure setup suitable for production or staging environments:

- **DynamoDB Configuration**:
  - Pay-per-request billing mode
  - Global Secondary Indexes (GSIs)
  - Point-in-time recovery
  - Server-side encryption

- **Caching with DAX**:
  - DAX cluster with 3 nodes
  - Subnet group configuration
  - Parameter group
  - IAM role with appropriate permissions

- **Timestream Database**:
  - Performance metrics table
  - Configurable retention periods

- **SQS Queues**:
  - FIFO queues for guaranteed ordering
  - Dead-letter queues for failed messages
  - Visibility timeout configuration

- **Lambda Functions**:
  - API Handler
  - Event Processor
  - Data Ingestion
  - Performance Calculator
  - x86_64 architecture

## Development Template (`template-development.yml`)

The development template provides a streamlined infrastructure setup suitable for development and testing:

- **DynamoDB Configuration**:
  - Basic table with minimal configuration
  - Point-in-time recovery
  - Server-side encryption

- **SQS Queues**:
  - Standard queue configuration
  - Dead-letter queue
  - Basic message retention

- **Lambda Functions**:
  - API Function
  - Event Processor Function
  - ARM64 architecture (cost-effective for development)

- **CloudWatch Alarms**:
  - API errors monitoring
  - Dead-letter queue monitoring

## Using the Templates

To specify which template to use with AWS SAM CLI:

```bash
# For production deployment
sam build --template template-production.yaml
sam deploy --template template-production.yaml --guided

# For development deployment
sam build --template template-development.yml
sam deploy --template template-development.yml --guided
```

## Choosing the Right Template

- Use `template-production.yaml` when:
  - Deploying to production or staging environments
  - Need for advanced features like DAX caching and Timestream
  - Requiring high availability and scalability

- Use `template-development.yml` when:
  - Deploying to development or testing environments
  - Need for cost-effective resources
  - Faster deployment and iteration cycles
  - Local testing and development

## Customizing Templates

When adding new resources, make sure to add them to the appropriate template based on their purpose. For resources needed in both environments, add them to both templates with appropriate configurations for each environment. 