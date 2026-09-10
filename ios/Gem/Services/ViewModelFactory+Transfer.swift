// Copyright (c). Gem Wallet. All rights reserved.

import FiatConnect
import Foundation
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Stake
import Store
import Swap
import SwiftUI
import Transfer
import WalletConnector
import WalletConnectorService
import class Gemstone.GemConfirmTransferService
import class Gemstone.GemFiatQuoteService
import class Gemstone.GemReceiveService
import class Gemstone.GemRecipientService
import class Gemstone.GemSwapQuoteService
import enum Gemstone.GemRecipientType
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult

extension ViewModelFactory {
    @MainActor
    public func validatorSelectScene(
        currentValidator: DelegationValidator?,
        recommended: [DelegationValidator],
        validators: [DelegationValidator],
        selectValidator: @escaping (DelegationValidator) -> Void,
    ) -> ValidatorSelectSceneViewModel {
        ValidatorSelectSceneViewModel(
            service: stakeService,
            currentValidator: currentValidator,
            recommended: recommended,
            validators: validators,
            selectValidator: selectValidator,
        )
    }

    @MainActor
    public func confirmTransferScene(
        wallet: Wallet,
        data: GemTransferData,
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction,
    ) -> ConfirmTransferSceneViewModel {
        return ConfirmTransferSceneViewModel(
            request: ConfirmTransferRequest(
                data: data,
                simulation: simulation,
                delegate: confirmTransferDelegate,
            ),
            wallet: wallet,
            confirmation: confirmTransferService().confirmation(wallet: wallet.toGem(), transfer: data, simulation: simulation),
            onComplete: { [toastPresenter] result in
                Task { toastPresenter.present(.transfer(result, for: data.inputType)) }
                onComplete?()
            },
        )
    }

    @MainActor
    public func paymentVerificationScene(
        verification: PaymentVerification,
        wallet: Wallet,
        onComplete: @escaping (PaymentDestination) -> Void,
    ) -> PaymentVerificationViewModel {
        PaymentVerificationViewModel(verification: verification, wallet: wallet, service: paymentService, onComplete: onComplete)
    }

    private func confirmTransferService() -> GemConfirmTransferService {
        GemConfirmTransferService(
            confirm: confirmService,
            explorer: explorerService,
            names: nameService,
            assetConfig: assetConfig,
            signer: KeystoreTransactionSigner(keystore: keystore),
            password: GemstoneKeystorePassword(keystore: keystore),
            recentActivity: recentAssetsService,
            preferences: preferencesService,
            payment: paymentService,
        )
    }

    @MainActor
    public func receiveScene(assetData: AssetData, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetData: assetData, wallet: wallet, service: receiveService())
    }

    @MainActor
    public func receiveScene(assetAddress: AssetAddress, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetAddress: assetAddress, wallet: wallet, service: receiveService())
    }

    private func receiveService() -> GemReceiveService {
        GemReceiveService(balances: balanceService, assets: assetsService)
    }

    @MainActor
    public func recipientScene(
        wallet: Wallet,
        asset: Asset,
        type: GemRecipientType,
        recipient: GemPaymentRecipient? = .none,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: GemRecipientService(names: nameService, payments: paymentService, session: walletSessionService),
            nameService: nameService,
            type: type,
            recipient: recipient,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func amountScene(
        input: AmountInput,
        wallet: Wallet,
        onTransferAction: TransferDataAction,
    ) -> AmountSceneViewModel {
        AmountSceneViewModel(
            input: input,
            wallet: wallet,
            service: amountService,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func fiatScene(
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            service: GemFiatQuoteService(fiat: fiatService, balances: balanceService, session: walletSessionService),
            assetAddress: assetAddress,
            wallet: wallet,
            type: type,
            amount: amount,
        )
    }

    @MainActor
    public func swapScene(
        input: SwapInput,
        onSwap: @escaping (GemTransferData) -> Void,
    ) -> SwapSceneViewModel {
        SwapSceneViewModel(
            service: GemSwapQuoteService(
                swap: swapService,
                preferences: preferencesService,
                balances: balanceService,
                stream: streamSubscriptionService,
                session: walletSessionService,
            ),
            input: input,
            onSwap: onSwap,
        )
    }

    @MainActor
    public func stakeScene(
        wallet: Wallet,
        chain: Chain,
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: StakeChain(rawValue: chain.rawValue)!, // Expected Only StakeChain accepted.
            service: stakeService,
        )
    }

    @MainActor
    public func earnScene(
        wallet: Wallet,
        asset: Asset,
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: stakeService,
        )
    }

    @MainActor
    public func delegationScene(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        validators: [DelegationValidator],
        onAmountInputAction: AmountInputAction,
        onTransferAction: TransferDataAction,
    ) -> DelegationSceneViewModel {
        DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(service: stakeService, delegation: delegation, asset: asset, formatter: .auto, currency: stakeService.getCurrency().toPrimitives()),
            asset: asset,
            service: stakeService,
            validators: validators,
            onAmountInputAction: onAmountInputAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func signMessageScene(
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            service: signMessageService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
