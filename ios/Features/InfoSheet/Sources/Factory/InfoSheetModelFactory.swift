// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public enum InfoSheetModelFactory {
    public static func create(from type: InfoSheetType) -> InfoSheetModel {
        switch type {
        case let .networkFee(asset):
            return InfoSheetModel(
                title: Localized.Info.NetworkFee.title,
                description: Localized.Info.NetworkFee.description(asset.chain.networkName.boldMarkdown(), asset.symbol.boldMarkdown()),
                image: .image(Images.Info.networkFee),
                button: .url(AppUrl.docs(.networkFees)),
            )
        case let .balanceRequired(info, image, button):
            return InfoSheetModel(
                title: Localized.Info.balanceRequiredTitle(info.title),
                description: Localized.Info.balanceRequiredDescription(
                    info.required.boldText,
                    info.available.boldText,
                    info.shortfall.boldText,
                ),
                image: .assetImage(image),
                button: button,
            )
        case let .insufficientNetworkFee(info, image, button):
            let description: String = if info.available != nil {
                Localized.Info.InsufficientNetworkFeeBalance.description(
                    info.requiredWithFiat.boldMarkdown(),
                    info.networkName.boldMarkdown(),
                    info.available.boldText,
                    info.shortfall.boldText,
                )
            } else {
                Localized.Transfer.insufficientNetworkFeeBalance(info.title.boldMarkdown())
            }
            return InfoSheetModel(
                title: Localized.Info.balanceRequiredTitle(info.title),
                description: description,
                image: .assetImage(image),
                button: button,
            )
        case let .transactionState(imageURL, placeholder, model):
            return InfoSheetModel(
                title: model.title,
                description: model.description,
                image: .assetImage(AssetImage(imageURL: imageURL, placeholder: placeholder, chainPlaceholder: model.stateImage)),
                button: .url(AppUrl.docs(.transactionStatus)),
            )
        case let .estimatedConfirmation(chain):
            return InfoSheetModel(
                title: Localized.Transaction.estimatedConfirmation,
                description: Localized.Info.estimatedConfirmationDescription(chain.networkName.boldMarkdown()),
                image: .image(Images.Info.networkFee),
            )
        case .watchWallet:
            return InfoSheetModel(
                title: Localized.Info.WatchWallet.title,
                description: Localized.Info.WatchWallet.description,
                image: .image(Images.Wallets.watch),
                button: .url(AppUrl.docs(.whatIsWatchWallet)),
            )
        case .paymentVerification:
            return InfoSheetModel(
                title: Localized.Info.paymentVerificationTitle,
                description: Localized.Info.paymentVerificationDescription,
                image: .image(Images.Logo.logo),
                button: nil,
            )
        case let .stakeLockTime(placeholder):
            return InfoSheetModel(
                title: Localized.Stake.lockTime,
                description: Localized.Info.LockTime.description,
                image: placeholder.map { .image($0) },
                button: .url(AppUrl.docs(.stakingLockTime)),
            )
        case let .stakeApr(placeholder):
            return InfoSheetModel(
                title: Localized.Stake.apr(""),
                description: Localized.Info.Stake.Apr.description,
                image: placeholder.map { .image($0) },
                button: .url(AppUrl.docs(.stakingApr)),
            )
        case .priceImpact:
            return InfoSheetModel(
                title: Localized.Swap.priceImpact,
                description: Localized.Info.PriceImpact.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.priceImpact)),
            )
        case .slippage:
            return InfoSheetModel(
                title: Localized.Swap.slippage,
                description: Localized.Info.Slippage.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.slippage)),
            )
        case .noQuote:
            return InfoSheetModel(
                title: Localized.Errors.Swap.noQuoteAvailable,
                description: Localized.Info.NoQuote.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.noQuotes)),
            )
        case let .assetStatus(status):
            return InfoSheetModel(
                title: status.statusTitle,
                description: status.statusDescription,
                image: .assetImage(status.statusAssetImage),
                button: .url(AppUrl.docs(.tokenVerification)),
            )
        case let .accountMinimalBalance(info):
            let amount = info.required?.text() ?? .empty
            return InfoSheetModel(
                title: Localized.Info.AccountMinimumBalance.title,
                description: Localized.Transfer.minimumAccountBalance(amount.boldMarkdown()),
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.accountMinimalBalance)),
            )
        case let .minimumAmount(asset, required, action):
            let formatter = ValueFormatter(style: .full)
            let chain = asset.chain.networkName.boldMarkdown()
            let amount = formatter.string(required, asset: asset).boldMarkdown()
            return InfoSheetModel(
                title: Localized.Info.MinimumAmount.title,
                description: Localized.Info.MinimumAmount.description(chain, amount),
                image: .image(Images.Logo.logo),
                button: action.map { .action(title: Localized.Asset.buyAsset(asset.symbol), action: $0) },
            )
        case let .swapMinimumAmount(info, providerName, image, button):
            return InfoSheetModel(
                title: Localized.Info.MinimumAmount.title,
                description: Localized.Info.swapMinimumAmountDescription(
                    providerName.boldMarkdown(),
                    info.requiredWithFiat.boldMarkdown(),
                    info.available.boldText,
                    info.shortfall.boldText,
                ),
                image: .assetImage(image),
                button: button,
            )
        case let .stakingReservedFees(image):
            return InfoSheetModel(
                title: Localized.Info.Stake.Reserved.title,
                description: Localized.Info.Stake.Reserved.description,
                image: .assetImage(image),
                button: .url(AppUrl.docs(.networkFees)),
            )
        case .pendingUnconfirmedBalance:
            return InfoSheetModel(
                title: Localized.Stake.pending,
                description: Localized.Info.Transaction.Pending.description,
                image: .image(Images.Logo.logo),
            )
        case .stakeFrozenRequired:
            return InfoSheetModel(
                title: Localized.Info.stakeFrozenRequiredTitle,
                description: Localized.Info.stakeFrozenRequiredDescription,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.staking(StakeChain.tron.rawValue))),
            )
        case .fundingApr:
            return InfoSheetModel(
                title: Localized.Info.Perpetual.FundingApr.title,
                description: Localized.Info.Perpetual.FundingApr.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.perpetualsFundingRate)),
            )
        case .fundingPayments:
            return InfoSheetModel(
                title: Localized.Info.Perpetual.FundingPayments.title,
                description: Localized.Info.Perpetual.FundingPayments.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.perpetualsFundingPayments)),
            )
        case .liquidationPrice:
            return InfoSheetModel(
                title: Localized.Info.Perpetual.LiquidationPrice.title,
                description: Localized.Info.Perpetual.LiquidationPrice.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.perpetualsLiquidationPrice)),
            )
        case .openInterest:
            return InfoSheetModel(
                title: Localized.Info.Perpetual.OpenInterest.title,
                description: Localized.Info.Perpetual.OpenInterest.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.perpetualsOpenInterest)),
            )
        case .autoclose:
            return InfoSheetModel(
                title: Localized.Perpetual.autoClose,
                description: Localized.Info.Perpetual.AutoClose.description,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.perpetualsAutoclose)),
            )
        case .maliciousTransaction:
            return InfoSheetModel(
                title: Localized.Errors.ScanTransaction.Malicious.title,
                description: Localized.Errors.ScanTransaction.Malicious.description,
                image: .image(Images.Logo.logo),
            )
        case let .memoRequired(symbol):
            return InfoSheetModel(
                title: Localized.Common.warning,
                description: Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown()),
                image: .image(Images.Logo.logo),
            )
        case let .dustThreshold(chain, image):
            return InfoSheetModel(
                title: Localized.Errors.transferError,
                description: Localized.Errors.dustThreshold(chain.networkName.boldMarkdown()),
                image: .assetImage(image),
                button: .url(AppUrl.docs(.dust)),
            )
        case .fullyDilutedValuation:
            return InfoSheetModel(
                title: Localized.Info.FullyDilutedValuation.title,
                description: Localized.Info.FullyDilutedValuation.description,
                image: .image(Images.Logo.logo),
            )
        case .circulatingSupply:
            return InfoSheetModel(
                title: Localized.Asset.circulatingSupply,
                description: Localized.Info.CirculatingSupply.description,
                image: .image(Images.Logo.logo),
            )
        case .totalSupply:
            return InfoSheetModel(
                title: Localized.Asset.totalSupply,
                description: Localized.Info.TotalSupply.description,
                image: .image(Images.Logo.logo),
            )
        case .maxSupply:
            return InfoSheetModel(
                title: Localized.Info.MaxSupply.title,
                description: Localized.Info.MaxSupply.description,
                image: .image(Images.Logo.logo),
            )
        }
    }
}
