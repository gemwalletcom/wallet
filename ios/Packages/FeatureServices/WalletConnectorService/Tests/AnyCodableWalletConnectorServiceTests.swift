import Foundation
import ReownWalletKit
import Testing
@testable import WalletConnectorService

struct AnyCodableWalletConnectorServiceTests {
    @Test
    func nullEncodesAsJsonNull() throws {
        #expect(try String(data: JSONEncoder().encode(AnyCodable.null()), encoding: .utf8) == "null")
    }
}
