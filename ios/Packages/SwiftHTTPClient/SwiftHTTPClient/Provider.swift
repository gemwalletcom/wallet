import Foundation

let encoder: JSONEncoder = {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    return encoder
}()

public struct Provider<T: TargetType>: Sendable {
    public typealias Target = T

    public let session: URLSession
    public let options: ProviderOptions<T>

    public init(
        session: URLSession = .shared,
        options: ProviderOptions<T> = ProviderOptions(baseUrl: nil),
    ) {
        self.session = session
        self.options = options
    }

    public func request(_ api: Target) async throws -> Response {
        var request = try TargetRequestBuilder(
            baseUrl: options.baseUrl ?? api.baseUrl,
            method: api.method,
            path: api.path,
            data: api.data,
            contentType: api.contentType,
            cachePolicy: api.cachePolicy,
            headers: api.headers,
        ).build(encoder: encoder)
        if let interceptor = options.requestInterceptor {
            try interceptor(&request, api)
        }
        let (data, response) = try await session.data(for: request, delegate: nil)
        return try .make(data: data, response: response)
    }
}
