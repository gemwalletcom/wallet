// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.PaymentInvoice
import Primitives

public struct PaymentVerification: Identifiable, Sendable {
    public let url: URL
    public let invoice: PaymentInvoice
    public let assetId: AssetId

    public var id: String { url.absoluteString }

    public init(url: String, invoice: PaymentInvoice, assetId: AssetId) throws {
        guard let url = URL(string: url) else {
            throw AnyError("Invalid payment verification URL")
        }
        self.url = url
        self.invoice = invoice
        self.assetId = assetId
    }
}
