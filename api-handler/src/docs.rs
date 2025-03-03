//! API documentation

/// # Pagination
/// 
/// Many API endpoints that return lists of items support pagination to limit the number of items
/// returned in a single response. These endpoints accept the following query parameters:
/// 
/// - `limit`: The maximum number of items to return in a single response.
/// - `next_token`: A token returned from a previous response to get the next page of results.
/// 
/// Example request:
/// ```
/// GET /api/portfolios?limit=10
/// ```
/// 
/// Example response:
/// ```json
/// {
///   "items": [...],
///   "next_token": "eyJpZCI6Imxhc3QtaXRlbS1pZCJ9"
/// }
/// ```
/// 
/// To get the next page of results, include the `next_token` from the previous response:
/// ```
/// GET /api/portfolios?limit=10&next_token=eyJpZCI6Imxhc3QtaXRlbS1pZCJ9
/// ```
/// 
/// When there are no more results, the response will not include a `next_token`. 