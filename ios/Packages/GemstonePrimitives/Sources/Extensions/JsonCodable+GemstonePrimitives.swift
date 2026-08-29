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

    func json() -> String {
        guard let data = try? JsonCodableEncoder.standard.encode(self) else {
            assertionFailure("failed to serialize \(Self.self)")
            return ""
        }
        return String(decoding: data, as: UTF8.self)
    }
}

extension Primitives.AccountDataType: JsonCodable {}
extension Primitives.AddressName: JsonCodable {}
extension Primitives.ApplicationMetadata: JsonCodable {}
extension Primitives.ApplicationMetadataSource: JsonCodable {}
extension Primitives.ApprovalData: JsonCodable {}
extension Primitives.Asset: JsonCodable {}
extension Primitives.AssetBasic: JsonCodable {}
extension Primitives.AssetFull: JsonCodable {}
extension Primitives.AssetList: JsonCodable {}
extension Primitives.AssetMarket: JsonCodable {}
extension Primitives.AssetPrice: JsonCodable {}
extension Primitives.AssetType: JsonCodable {}
extension Primitives.AuthNonce: JsonCodable {}
extension Primitives.AuthPayload: JsonCodable {}
extension Primitives.BalanceMetadata: JsonCodable {}
extension Primitives.BannerEvent: JsonCodable {}
extension Primitives.BannerState: JsonCodable {}
extension Primitives.CancelOrderData: JsonCodable {}
extension Primitives.ChainAddress: JsonCodable {}
extension Primitives.ChainAsset: JsonCodable {}
extension Primitives.Appearance: JsonCodable {}
extension Primitives.ChartCandleStick: JsonCodable {}
extension Primitives.ChartCandleUpdate: JsonCodable {}
extension Primitives.ChartDateValue: JsonCodable {}
extension Primitives.ChartValuePercentage: JsonCodable {}
extension Primitives.ChartPeriod: JsonCodable {}
extension Primitives.Charts: JsonCodable {}
extension Primitives.ConfigResponse: JsonCodable {}
extension Primitives.ConfigVersions: JsonCodable {}
extension Primitives.ConnectionComponent: JsonCodable {}
extension Primitives.ConnectionStatus: JsonCodable {}
extension Primitives.Contact: JsonCodable {}
extension Primitives.ContactAddress: JsonCodable {}
extension Primitives.ContractCallData: JsonCodable {}
extension Primitives.Currency: JsonCodable {}
extension Primitives.Delegation: JsonCodable {}
extension Primitives.DelegationBase: JsonCodable {}
extension Primitives.DelegationState: JsonCodable {}
extension Primitives.DelegationValidator: JsonCodable {}
extension Primitives.Device: JsonCodable {}
extension Primitives.EarnType: JsonCodable {}
extension Primitives.FiatAssets: JsonCodable {}
extension Primitives.FiatQuote: JsonCodable {}
extension Primitives.FiatQuoteRequest: JsonCodable {}
extension Primitives.FiatQuoteType: JsonCodable {}
extension Primitives.FiatQuoteUrl: JsonCodable {}
extension Primitives.FiatQuotes: JsonCodable {}
extension Primitives.FiatTransactionData: JsonCodable {}
extension Primitives.InAppNotification: JsonCodable {}
extension Primitives.Markets: JsonCodable {}
extension Primitives.NFTAsset: JsonCodable {}
extension Primitives.NFTAssetData: JsonCodable {}
extension Primitives.NFTAttribute: JsonCodable {}
extension Primitives.NFTAttributeType: JsonCodable {}
extension Primitives.NFTData: JsonCodable {}
extension Primitives.NFTImages: JsonCodable {}
extension Primitives.NFTResource: JsonCodable {}
extension Primitives.NFTType: JsonCodable {}
extension Primitives.NameRecord: JsonCodable {}
extension Primitives.Node: JsonCodable {}
extension Primitives.TransactionId: JsonCodable {}
extension Primitives.Wallet: JsonCodable {}
extension Primitives.FiatRate: JsonCodable {}
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
extension Primitives.PlatformStore: JsonCodable {}
extension Primitives.Release: JsonCodable {}
extension Primitives.PerpetualPosition: JsonCodable {}
extension Primitives.PerpetualPositionsSummary: JsonCodable {}
extension Primitives.PerpetualReduceData: JsonCodable {}
extension Primitives.PerpetualTriggerOrder: JsonCodable {}
extension Primitives.PerpetualType: JsonCodable {}
extension Primitives.PortfolioAssets: JsonCodable {}
extension Primitives.PortfolioAsset: JsonCodable {}
extension Primitives.PortfolioAssetsRequest: JsonCodable {}
extension Primitives.PortfolioData: JsonCodable {}
extension Primitives.Price: JsonCodable {}
extension Primitives.PriceAlert: JsonCodable {}
extension Primitives.PriceAlertNotificationType: JsonCodable {}
extension Primitives.RedemptionRequest: JsonCodable {}
extension Primitives.RedemptionResult: JsonCodable {}
extension Primitives.ReferralCode: JsonCodable {}
extension Primitives.ReportNft: JsonCodable {}
extension Primitives.Resource: JsonCodable {}
extension Primitives.Rewards: JsonCodable {}
extension Primitives.ScanAddressTarget: JsonCodable {}
extension Primitives.ScanTransaction: JsonCodable {}
extension Primitives.ScanTransactionPayload: JsonCodable {}
extension Primitives.SearchResponse: JsonCodable {}
extension Primitives.SimulationPayloadField: JsonCodable {}
extension Primitives.SimulationResult: JsonCodable {}
extension Primitives.SimulationHeader: JsonCodable {}
extension Primitives.SolanaNftStandard: JsonCodable {}
extension Primitives.SolanaTokenProgramId: JsonCodable {}
extension Primitives.StakeProviderType: JsonCodable {}
extension Primitives.StakeType: JsonCodable {}
extension Primitives.StakeValidator: JsonCodable {}
extension Primitives.SupportMessage: JsonCodable {}
extension Primitives.SupportMessageInput: JsonCodable {}
extension Primitives.SupportTyping: JsonCodable {}
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
extension Primitives.TransactionsResponse: JsonCodable {}
extension Primitives.TransferDataOutputAction: JsonCodable {}
extension Primitives.TransferDataOutputType: JsonCodable {}
extension Primitives.TronStakeData: JsonCodable {}
extension Primitives.TronUnfreeze: JsonCodable {}
extension Primitives.TronVote: JsonCodable {}
extension Primitives.UTXO: JsonCodable {}
extension Primitives.WalletConfigurationResult: JsonCodable {}
extension Primitives.Platform: JsonCodable {}
extension Primitives.DeviceLocale: JsonCodable {}
extension Primitives.WalletConnection: JsonCodable {}
extension Primitives.WalletConnectionSession: JsonCodable {}
extension Primitives.WalletConnectionSessionProposal: JsonCodable {}
extension Primitives.WalletSubscription: JsonCodable {}
extension Primitives.WalletSubscriptionChains: JsonCodable {}
