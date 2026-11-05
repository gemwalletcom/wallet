// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemLightningServiceProtocol
import class Gemstone.GemLightningWallet
import struct Gemstone.LightningInvoice
import struct Gemstone.LightningPayment
import Primitives

public final class GemLightningServiceMock: GemLightningServiceProtocol, @unchecked Sendable {
    public let wallet: GemLightningWalletMock
    public var openError: (any Error)?

    public private(set) var openCalls: [WalletId] = []
    public private(set) var openEnabledCalls: [WalletId] = []
    public private(set) var syncCalls: [WalletId] = []
    public private(set) var lockAllCallsCount = 0

    public init(wallet: GemLightningWalletMock = GemLightningWalletMock()) {
        self.wallet = wallet
    }

    public func lockAll() {
        lockAllCallsCount += 1
    }

    public func open(walletId: WalletId) async throws -> GemLightningWallet {
        openCalls.append(walletId)
        if let openError {
            throw openError
        }
        return wallet
    }

    public func openEnabled(walletId: WalletId) async throws -> Bool {
        openEnabledCalls.append(walletId)
        return true
    }

    public func sync(walletId: WalletId) async throws {
        syncCalls.append(walletId)
    }
}

public final class GemLightningWalletMock: GemLightningWallet, @unchecked Sendable {
    public var invoice: LightningInvoice
    public var payments: [LightningPayment] = []
    public var createInvoiceError: (any Error)?

    public private(set) var createInvoiceAmounts: [UInt64?] = []
    public private(set) var syncCallsCount = 0

    public init(
        invoice: LightningInvoice = LightningInvoice(
            invoice: "lnbc1invoice",
            network: .mainnet,
            amountSats: nil,
            paymentHash: "payment-hash",
            createdAt: 0,
            expiresAt: 4_102_444_800,
        ),
    ) {
        self.invoice = invoice
        super.init(noHandle: NoHandle())
    }

    public required init(unsafeFromHandle handle: UInt64) {
        invoice = LightningInvoice(invoice: "", network: .mainnet, amountSats: nil, paymentHash: "", createdAt: 0, expiresAt: 0)
        super.init(unsafeFromHandle: handle)
    }

    override public func createInvoice(amountSats: UInt64?, expirySecs _: UInt32?) async throws -> LightningInvoice {
        createInvoiceAmounts.append(amountSats)
        if let createInvoiceError {
            throw createInvoiceError
        }
        var invoice = invoice
        invoice.amountSats = amountSats
        return invoice
    }

    override public func payments(limit _: UInt64, offset _: UInt64) async throws -> [LightningPayment] {
        payments
    }

    override public func sync() async throws {
        syncCallsCount += 1
    }
}
