// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftHTTPClient

public enum GemDeviceAPI: TargetType {
    case getDevice
    case addDevice(device: Device)
    case updateDevice(device: Device)
    case isDeviceRegistered

    case getSubscriptions
    case addSubscriptions(subscriptions: [WalletSubscription])
    case deleteSubscriptions(subscriptions: [WalletSubscriptionChains])

    case getPriceAlerts(assetId: String?)
    case addPriceAlerts(priceAlerts: [PriceAlert])
    case deletePriceAlerts(priceAlerts: [PriceAlert])

    case getTransactions(walletId: WalletId, assetId: String?, fromTimestamp: Int)
    case getTransaction(transactionId: TransactionId)
    case getAssetsList(walletId: WalletId, fromTimestamp: Int)
    case getDeviceNFTAssets(walletId: WalletId)
    case getDeviceNFTAsset(assetId: NFTAssetId)
    case refreshNftAsset(walletId: WalletId, assetId: NFTAssetId)

    case reportNft(report: ReportNft)
    case scanTransaction(payload: ScanTransactionPayload)
    case getWalletConfiguration(walletId: WalletId)

    case getAuthNonce
    case getDeviceToken

    case getDeviceRewards(walletId: WalletId)
    case getDeviceRewardsEvents(walletId: WalletId)
    case createDeviceReferral(walletId: WalletId, request: AuthenticatedRequest<ReferralCode>)
    case useDeviceReferralCode(walletId: WalletId, request: AuthenticatedRequest<ReferralCode>)
    case redeemDeviceRewards(walletId: WalletId, request: AuthenticatedRequest<RedemptionRequest>)

    case getNotifications(fromTimestamp: Int)
    case markNotificationsRead

    case getFiatAssets(FiatQuoteType)
    case getFiatQuotes(walletId: WalletId, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest)
    case getFiatQuoteUrl(walletId: WalletId, quoteId: String)
    case getFiatTransactions(walletId: WalletId)

    case getNameRecord(name: String, chain: String)
    case getAddressNames(requests: [ChainAddress])

    case getPortfolioAssets(period: ChartPeriod, request: PortfolioAssetsRequest)

    public var baseUrl: URL {
        Constants.apiURL
    }

    public var method: HTTPMethod {
        switch self {
        case .getDevice,
             .getSubscriptions,
             .getTransactions,
             .getTransaction,
             .getAssetsList,
             .getPriceAlerts,
             .getDeviceNFTAssets,
             .getDeviceNFTAsset,
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getDeviceRewardsEvents,
             .getNotifications,
             .isDeviceRegistered,
             .getFiatAssets,
             .getFiatQuotes,
             .getFiatQuoteUrl,
             .getFiatTransactions,
             .getNameRecord,
             .getWalletConfiguration:
            .GET
        case .addDevice,
             .addSubscriptions,
             .addPriceAlerts,
             .scanTransaction,
             .refreshNftAsset,
             .reportNft,
             .createDeviceReferral,
             .useDeviceReferralCode,
             .redeemDeviceRewards,
             .markNotificationsRead,
             .getPortfolioAssets,
             .getAddressNames:
            .POST
        case .updateDevice:
            .PUT
        case .deleteSubscriptions,
             .deletePriceAlerts:
            .DELETE
        }
    }

    public var path: String {
        switch self {
        case .addDevice,
             .getDevice,
             .updateDevice:
            return "/v2/devices"
        case .isDeviceRegistered:
            return "/v2/devices/is_registered"
        case .getSubscriptions,
             .addSubscriptions,
             .deleteSubscriptions:
            return "/v2/devices/subscriptions"
        case .getPriceAlerts,
             .addPriceAlerts,
             .deletePriceAlerts:
            return "/v2/devices/price_alerts"
        case let .getTransactions(_, assetId, fromTimestamp):
            var path = "/v2/devices/transactions?from_timestamp=\(fromTimestamp)"
            if let assetId {
                path += "&asset_id=\(assetId)"
            }
            return path
        case let .getTransaction(transactionId):
            return "/v2/devices/transaction/\(transactionId.identifier)"
        case let .getAssetsList(_, fromTimestamp):
            return "/v2/devices/assets?from_timestamp=\(fromTimestamp)"
        case .getDeviceNFTAssets:
            return "/v2/devices/nft_assets"
        case let .getDeviceNFTAsset(assetId):
            return "/v2/devices/nft_assets/\(assetId.identifier)"
        case let .refreshNftAsset(_, assetId):
            return "/v2/devices/nft_assets/\(assetId.identifier)/refresh"
        case .reportNft:
            return "/v2/devices/nft/report"
        case .scanTransaction:
            return "/v2/devices/scan/transaction"
        case .getWalletConfiguration:
            return "/v2/devices/wallet_configuration"
        case .getAuthNonce:
            return "/v2/devices/auth/nonce"
        case .getDeviceToken:
            return "/v2/devices/token"
        case .getDeviceRewards:
            return "/v2/devices/rewards"
        case .getDeviceRewardsEvents:
            return "/v2/devices/rewards/events"
        case .createDeviceReferral:
            return "/v2/devices/rewards/referrals/create"
        case .useDeviceReferralCode:
            return "/v2/devices/rewards/referrals/use"
        case .redeemDeviceRewards:
            return "/v2/devices/rewards/redeem"
        case let .getNotifications(fromTimestamp):
            return "/v2/devices/notifications?from_timestamp=\(fromTimestamp)"
        case .markNotificationsRead:
            return "/v2/devices/notifications/read"
        case let .getFiatAssets(type):
            return "/v2/devices/fiat/assets/\(type.rawValue)"
        case let .getFiatQuotes(_, type, assetId, _):
            return "/v2/devices/fiat/quotes/\(type.rawValue)/\(assetId.identifier)"
        case let .getFiatQuoteUrl(_, quoteId):
            return "/v2/devices/fiat/quotes/\(quoteId)/url"
        case .getFiatTransactions:
            return "/v2/devices/fiat/transactions"
        case let .getNameRecord(name, chain):
            return "/v2/devices/name/resolve/\(name)?chain=\(chain)"
        case .getAddressNames:
            return "/v2/devices/address_names"
        case let .getPortfolioAssets(period, _):
            return "/v2/devices/portfolio/assets?period=\(period.rawValue)"
        }
    }

    public var walletId: String? {
        switch self {
        case let .getTransactions(walletId, _, _),
             let .getAssetsList(walletId, _),
             let .getDeviceNFTAssets(walletId),
             let .refreshNftAsset(walletId, _),
             let .getDeviceRewards(walletId),
             let .getDeviceRewardsEvents(walletId),
             let .createDeviceReferral(walletId, _),
             let .useDeviceReferralCode(walletId, _),
             let .redeemDeviceRewards(walletId, _),
             let .getFiatQuotes(walletId, _, _, _),
             let .getFiatQuoteUrl(walletId, _),
             let .getFiatTransactions(walletId),
             let .getWalletConfiguration(walletId):
            walletId.id
        default:
            nil
        }
    }

    public var data: RequestData {
        switch self {
        case .getDevice,
             .getSubscriptions,
             .getAssetsList,
             .getDeviceNFTAssets,
             .getDeviceNFTAsset,
             .refreshNftAsset,
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getDeviceRewardsEvents,
             .getNotifications,
             .markNotificationsRead,
             .getTransactions,
             .getTransaction,
             .getWalletConfiguration,
             .isDeviceRegistered,
             .getFiatAssets,
             .getFiatQuoteUrl,
             .getFiatTransactions,
             .getNameRecord:
            return .plain
        case let .getPriceAlerts(assetId):
            let params: [String: Any] = [
                "asset_id": assetId,
            ].compactMapValues { $0 }
            return .params(params)
        case let .getFiatQuotes(_, _, _, request):
            let params: [String: Any] = [
                "amount": request.amount,
                "currency": request.currency,
            ]
            return .params(params)
        case let .addDevice(device),
             let .updateDevice(device):
            return .encodable(device)
        case let .addSubscriptions(subscriptions):
            return .encodable(subscriptions)
        case let .deleteSubscriptions(subscriptions):
            return .encodable(subscriptions)
        case let .addPriceAlerts(priceAlerts),
             let .deletePriceAlerts(priceAlerts):
            return .encodable(priceAlerts)
        case let .scanTransaction(payload):
            return .encodable(payload)
        case let .reportNft(report):
            return .encodable(report)
        case let .createDeviceReferral(_, request):
            return .encodable(request)
        case let .useDeviceReferralCode(_, request):
            return .encodable(request)
        case let .redeemDeviceRewards(_, request):
            return .encodable(request)
        case let .getPortfolioAssets(_, request):
            return .encodable(request)
        case let .getAddressNames(requests):
            return .encodable(requests)
        }
    }
}
