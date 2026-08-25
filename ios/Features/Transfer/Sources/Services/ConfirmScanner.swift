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
        guard let payload = try? payload.map() else {
            return nil
        }
        return await scanService.getScanTransaction(payload: payload)?.map()
    }
}
