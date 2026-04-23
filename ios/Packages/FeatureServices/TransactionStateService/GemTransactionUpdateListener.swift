// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import Primitives

public final class GemTransactionUpdateListener: Gemstone.TransactionUpdateListener {
    private let processor: TransactionStateProcessor

    public init(processor: TransactionStateProcessor) {
        self.processor = processor
    }

    public func onUpdate(id: String, update: Gemstone.TransactionUpdate) async {
        do {
            try await processor.process(id: id, changes: update.map())
        } catch {
            debugLog("GemTransactionUpdateListener: \(id) failed to map update: \(error)")
        }
    }
}
