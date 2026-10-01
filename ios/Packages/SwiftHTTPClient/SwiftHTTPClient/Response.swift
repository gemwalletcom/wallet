import Foundation

public struct Response {
    public let body: Data

    public init(body: Data) {
        self.body = body
    }

    public static func make(data: Data, response urlResponse: URLResponse?) throws -> Response {
        guard urlResponse is HTTPURLResponse else {
            throw URLError(.badServerResponse)
        }
        return Response(body: data)
    }

    public func map<T: Decodable>(as type: T.Type, decoder: JSONDecoder) throws -> T {
        try decoder.decode(type, from: body)
    }
}
