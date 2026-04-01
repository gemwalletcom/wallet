// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftHTTPClient

public enum GemDeviceAPI: TargetType {
    case getDevice
    case addDevice(device: Device)
    case updateDevice(device: Device)
    case isDeviceRegistered
    case migrateDevice(request: MigrateDeviceIdRequest)

    case getSubscriptions
    case addSubscriptions(subscriptions: [WalletSubscription])
    case deleteSubscriptions(subscriptions: [WalletSubscriptionChains])

    case getPriceAlerts(assetId: String?)
    case addPriceAlerts(priceAlerts: [PriceAlert])
    case deletePriceAlerts(priceAlerts: [PriceAlert])

    case getTransactions(walletId: String, assetId: String?, fromTimestamp: Int)
    case getAssetsList(walletId: String, fromTimestamp: Int)
    case getDeviceNFTAssets(walletId: String)

    case reportNft(report: ReportNft)
    case scanTransaction(payload: ScanTransactionPayload)

    case getAuthNonce
    case getDeviceToken

    case getDeviceRewards(walletId: String)
    case getDeviceRewardsEvents(walletId: String)
    case getDeviceRedemptionOption(code: String)
    case createDeviceReferral(walletId: String, request: AuthenticatedRequest<ReferralCode>)
    case useDeviceReferralCode(walletId: String, request: AuthenticatedRequest<ReferralCode>)
    case redeemDeviceRewards(walletId: String, request: AuthenticatedRequest<RedemptionRequest>)

    case getNotifications(fromTimestamp: Int)
    case markNotificationsRead

    case getFiatAssets(FiatQuoteType)
    case getFiatQuotes(walletId: String, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest)
    case getFiatQuoteUrl(walletId: String, quoteId: String)
    case getFiatTransactions(walletId: String)

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
             .getAssetsList,
             .getPriceAlerts,
             .getDeviceNFTAssets,
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getDeviceRewardsEvents,
             .getDeviceRedemptionOption,
             .getNotifications,
             .isDeviceRegistered,
             .getFiatAssets,
             .getFiatQuotes,
             .getFiatQuoteUrl,
             .getFiatTransactions,
             .getNameRecord:
            .GET
        case .addDevice,
             .addSubscriptions,
             .addPriceAlerts,
             .scanTransaction,
             .reportNft,
             .migrateDevice,
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
        case .migrateDevice:
            return "/v2/devices/migrate"
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
        case let .getAssetsList(_, fromTimestamp):
            return "/v2/devices/assets?from_timestamp=\(fromTimestamp)"
        case .getDeviceNFTAssets:
            return "/v2/devices/nft_assets"
        case .reportNft:
            return "/v2/devices/nft/report"
        case .scanTransaction:
            return "/v2/devices/scan/transaction"
        case .getAuthNonce:
            return "/v2/devices/auth/nonce"
        case .getDeviceToken:
            return "/v2/devices/token"
        case .getDeviceRewards:
            return "/v2/devices/rewards"
        case .getDeviceRewardsEvents:
            return "/v2/devices/rewards/events"
        case let .getDeviceRedemptionOption(code):
            return "/v2/devices/rewards/redemptions/\(code)"
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

    public var headers: [String: String] {
        [:]
    }

    public var walletId: String? {
        switch self {
        case let .getTransactions(walletId, _, _),
             let .getAssetsList(walletId, _),
             let .getDeviceNFTAssets(walletId),
             let .getDeviceRewards(walletId),
             let .getDeviceRewardsEvents(walletId),
             let .createDeviceReferral(walletId, _),
             let .useDeviceReferralCode(walletId, _),
             let .redeemDeviceRewards(walletId, _),
             let .getFiatQuotes(walletId, _, _, _),
             let .getFiatQuoteUrl(walletId, _),
             let .getFiatTransactions(walletId):
            walletId
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
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getDeviceRewardsEvents,
             .getDeviceRedemptionOption,
             .getNotifications,
             .markNotificationsRead,
             .getTransactions,
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
        case let .migrateDevice(request):
            return .encodable(request)
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
