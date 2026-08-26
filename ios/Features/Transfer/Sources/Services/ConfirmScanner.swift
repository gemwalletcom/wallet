// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import ScanService

public final class ConfirmScanner: GemConfirmScanner {
    private let scanService: ScanService

    public init(scanService: ScanService) {
        self.scanService = scanService
    }

    public func scanTransaction(payload: Gemstone.ScanTransactionPayload) async -> Gemstone.ScanTransaction? {
        guard let payload = try? Primitives.ScanTransactionPayload(payload) else {
            return nil
        }
        return try? await scanService.getScanTransaction(payload: payload)?.json()
    }
}
