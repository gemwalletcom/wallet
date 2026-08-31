// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemDeviceRequestSignerProtocol

public extension GemDeviceRequestSignerProtocol {
    func sign(request: inout URLRequest, walletId: String = "") throws {
        let header = try sign(
            method: request.httpMethod ?? "GET",
            path: request.url?.path ?? "/",
            walletId: walletId,
            body: request.httpBody ?? Data(),
        )
        request.setValue(header, forHTTPHeaderField: "Authorization")
    }
}
