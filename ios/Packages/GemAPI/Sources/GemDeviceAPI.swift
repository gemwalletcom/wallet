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
    case getAssetsList(walletId: WalletId, fromTimestamp: Int)

    case getWalletConfiguration(walletId: WalletId)

    case getSupportMessages(fromTimestamp: Int)
    case sendSupportMessage(input: SupportMessageInput)
    case sendSupportImage(image: Data, fileName: String, mimeType: String)

    case getAuthNonce
    case getDeviceToken

    case getDeviceRewards(walletId: WalletId)
    case createDeviceReferral(walletId: WalletId, request: AuthenticatedRequest<ReferralCode>)
    case useDeviceReferralCode(walletId: WalletId, request: AuthenticatedRequest<ReferralCode>)
    case redeemDeviceRewards(walletId: WalletId, request: AuthenticatedRequest<RedemptionRequest>)

    case getNotifications(fromTimestamp: Int)
    case markNotificationsRead

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
             .getAssetsList,
             .getPriceAlerts,
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getNotifications,
             .isDeviceRegistered,
             .getFiatQuotes,
             .getFiatQuoteUrl,
             .getFiatTransactions,
             .getNameRecord,
             .getWalletConfiguration,
             .getSupportMessages:
            .GET
        case .addDevice,
             .addSubscriptions,
             .addPriceAlerts,
             .createDeviceReferral,
             .useDeviceReferralCode,
             .redeemDeviceRewards,
             .markNotificationsRead,
             .getPortfolioAssets,
             .getAddressNames,
             .sendSupportMessage,
             .sendSupportImage:
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
        case let .getAssetsList(_, fromTimestamp):
            return "/v2/devices/assets?from_timestamp=\(fromTimestamp)"
        case .getWalletConfiguration:
            return "/v2/devices/wallet_configuration"
        case let .getSupportMessages(fromTimestamp):
            return "/v2/devices/support/messages?from_timestamp=\(fromTimestamp)"
        case .sendSupportMessage:
            return "/v2/devices/support/messages"
        case let .sendSupportImage(_, fileName, _):
            let encodedFileName = fileName.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? fileName
            return "/v2/devices/support/messages/images?file_name=\(encodedFileName)"
        case .getAuthNonce:
            return "/v2/devices/auth/nonce"
        case .getDeviceToken:
            return "/v2/devices/token"
        case .getDeviceRewards:
            return "/v2/devices/rewards"
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
             let .getDeviceRewards(walletId),
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
             .getAuthNonce,
             .getDeviceToken,
             .getDeviceRewards,
             .getNotifications,
             .markNotificationsRead,
             .getTransactions,
             .getWalletConfiguration,
             .isDeviceRegistered,
             .getFiatQuoteUrl,
             .getFiatTransactions,
             .getNameRecord,
             .getSupportMessages:
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
        case let .sendSupportMessage(input):
            return .encodable(input)
        case let .sendSupportImage(image, _, _):
            return .data(image)
        }
    }

    public var contentType: String {
        switch self {
        case let .sendSupportImage(_, _, mimeType):
            mimeType
        default:
            ContentType.json.rawValue
        }
    }
}
