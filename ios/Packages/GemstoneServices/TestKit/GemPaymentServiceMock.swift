// Copyright (c). Gem Wallet. All rights reserved.

public import typealias Gemstone.AssetId
public import struct Gemstone.Asset
public import struct Gemstone.ChainAddress
public import struct Gemstone.GemPaymentConfirmTransfer
public import enum Gemstone.GemPaymentDestination
public import enum Gemstone.GemPaymentLoad
public import class Gemstone.GemPaymentService
public import protocol Gemstone.GemPaymentServiceProtocol
public import struct Gemstone.GemPaymentWalletAsset
public import struct Gemstone.GemTransferData
public import enum Gemstone.Payment
public import struct Gemstone.PaymentInvoice
public import enum Gemstone.PaymentLink
public import struct Gemstone.PaymentRequest
import GemstonePrimitivesTestKit
import Primitives

public final class GemPaymentServiceMock: GemPaymentServiceProtocol, @unchecked Sendable {
    private let service = GemPaymentService.mock()
    private let loadResult: Result<GemPaymentLoad, any Error>
    public private(set) var selectedAssetIds: [AssetId] = []

    public init(
        load: Result<GemPaymentLoad, any Error> = .failure(AnyError("not stubbed")),
    ) {
        loadResult = load
    }

    public func load(link _: PaymentLink, addresses _: [ChainAddress]) async throws -> GemPaymentLoad {
        try loadResult.get()
    }

    public func selectAsset(invoice _: PaymentInvoice, addresses _: [ChainAddress], assetId: AssetId) async throws -> GemPaymentLoad {
        selectedAssetIds.append(assetId)
        return try loadResult.get()
    }

    public func decodeUrl(string: String) throws -> Payment {
        try service.decodeUrl(string: string)
    }

    public func destination(request: PaymentRequest, assets: [GemPaymentWalletAsset]) -> GemPaymentDestination {
        service.destination(request: request, assets: assets)
    }

    public func transferDestination(request: PaymentRequest, asset: GemPaymentWalletAsset) -> GemPaymentDestination {
        service.transferDestination(request: request, asset: asset)
    }

    public func transferData(transfer: GemPaymentConfirmTransfer, asset: Asset) -> GemTransferData {
        service.transferData(transfer: transfer, asset: asset)
    }
}
