// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

private enum JsonCodableEncoder {
    static let standard: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }()
}

public protocol JsonCodable: Codable {}

public extension JsonCodable {
    init(_ json: String) throws {
        self = try JSONDateDecoder.standard.decode(Self.self, from: Data(json.utf8))
    }

    func json() throws -> String {
        try String(decoding: JsonCodableEncoder.standard.encode(self), as: UTF8.self)
    }
}

extension Primitives.AccountDataType: JsonCodable {}
extension Primitives.ApplicationMetadata: JsonCodable {}
extension Primitives.ApplicationMetadataSource: JsonCodable {}
extension Primitives.ApprovalData: JsonCodable {}
extension Primitives.Asset: JsonCodable {}
extension Primitives.AssetType: JsonCodable {}
extension Primitives.BalanceMetadata: JsonCodable {}
extension Primitives.CancelOrderData: JsonCodable {}
extension Primitives.ChainAsset: JsonCodable {}
extension Primitives.ChartCandleStick: JsonCodable {}
extension Primitives.ChartCandleUpdate: JsonCodable {}
extension Primitives.ChartDateValue: JsonCodable {}
extension Primitives.ChartPeriod: JsonCodable {}
extension Primitives.Charts: JsonCodable {}
extension Primitives.ConnectionComponent: JsonCodable {}
extension Primitives.ConnectionStatus: JsonCodable {}
extension Primitives.ContractCallData: JsonCodable {}
extension Primitives.Delegation: JsonCodable {}
extension Primitives.DelegationBase: JsonCodable {}
extension Primitives.DelegationState: JsonCodable {}
extension Primitives.DelegationValidator: JsonCodable {}
extension Primitives.EarnType: JsonCodable {}
extension Primitives.NFTAsset: JsonCodable {}
extension Primitives.NFTAttribute: JsonCodable {}
extension Primitives.NFTAttributeType: JsonCodable {}
extension Primitives.NFTImages: JsonCodable {}
extension Primitives.NFTResource: JsonCodable {}
extension Primitives.NFTType: JsonCodable {}
extension Primitives.Payment: JsonCodable {}
extension Primitives.PaymentAmount: JsonCodable {}
extension Primitives.PaymentLink: JsonCodable {}
extension Primitives.PaymentRequest: JsonCodable {}
extension Primitives.PerpetualAccountMode: JsonCodable {}
extension Primitives.PerpetualAccountSummary: JsonCodable {}
extension Primitives.PerpetualBalance: JsonCodable {}
extension Primitives.PerpetualConfirmData: JsonCodable {}
extension Primitives.PerpetualData: JsonCodable {}
extension Primitives.PerpetualDirection: JsonCodable {}
extension Primitives.PerpetualMarginType: JsonCodable {}
extension Primitives.PerpetualMarketData: JsonCodable {}
extension Primitives.PerpetualMetadata: JsonCodable {}
extension Primitives.PerpetualModifyConfirmData: JsonCodable {}
extension Primitives.PerpetualModifyPositionType: JsonCodable {}
extension Primitives.PerpetualOrderType: JsonCodable {}
extension Primitives.PerpetualPortfolio: JsonCodable {}
extension Primitives.PerpetualPortfolioTimeframeData: JsonCodable {}
extension Primitives.PerpetualPosition: JsonCodable {}
extension Primitives.PerpetualPositionsSummary: JsonCodable {}
extension Primitives.PerpetualReduceData: JsonCodable {}
extension Primitives.PerpetualTriggerOrder: JsonCodable {}
extension Primitives.PerpetualType: JsonCodable {}
extension Primitives.Price: JsonCodable {}
extension Primitives.Resource: JsonCodable {}
extension Primitives.ScanAddressTarget: JsonCodable {}
extension Primitives.ScanTransaction: JsonCodable {}
extension Primitives.ScanTransactionPayload: JsonCodable {}
extension Primitives.SimulationPayloadField: JsonCodable {}
extension Primitives.SimulationResult: JsonCodable {}
extension Primitives.SolanaNftStandard: JsonCodable {}
extension Primitives.SolanaTokenProgramId: JsonCodable {}
extension Primitives.StakeProviderType: JsonCodable {}
extension Primitives.StakeValidator: JsonCodable {}
extension Primitives.StakeType: JsonCodable {}
extension Primitives.SwapData: JsonCodable {}
extension Primitives.SwapPriceImpact: JsonCodable {}
extension Primitives.SwapPriceImpactType: JsonCodable {}
extension Primitives.SwapProviderData: JsonCodable {}
extension Primitives.SwapQuote: JsonCodable {}
extension Primitives.SwapQuoteData: JsonCodable {}
extension Primitives.SwapQuoteDataType: JsonCodable {}
extension Primitives.TPSLOrderData: JsonCodable {}
extension Primitives.Transaction: JsonCodable {}
extension Primitives.TransactionPerpetualMetadata: JsonCodable {}
extension Primitives.TransactionState: JsonCodable {}
extension Primitives.TransactionType: JsonCodable {}
extension Primitives.TransferDataOutputAction: JsonCodable {}
extension Primitives.TransferDataOutputType: JsonCodable {}
extension Primitives.TronStakeData: JsonCodable {}
extension Primitives.TronUnfreeze: JsonCodable {}
extension Primitives.TronVote: JsonCodable {}
extension Primitives.UTXO: JsonCodable {}
