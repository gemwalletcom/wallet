// Copyright (c). Gem Wallet. All rights reserved.

import FiatConnect
import Foundation
import class Gemstone.GemConfirmTransferService
import class Gemstone.GemFiatQuoteService
import struct Gemstone.GemPaymentRecipient
import class Gemstone.GemReceiveService
import class Gemstone.GemRecipientService
import enum Gemstone.GemRecipientType
import class Gemstone.GemSwapQuoteService
import struct Gemstone.GemTransferData
import struct Gemstone.GemValidatorRow
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.SimulationResult
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

public extension ViewModelFactory {
    @MainActor
    func validatorSelectScene(
        currentValidator: DelegationValidator?,
        recommended: [GemValidatorRow],
        validators: [GemValidatorRow],
        selectValidator: @escaping (GemValidatorRow) -> Void,
    ) -> ValidatorSelectSceneViewModel {
        ValidatorSelectSceneViewModel(
            currentValidator: currentValidator,
            recommended: recommended,
            validators: validators,
            selectValidator: selectValidator,
        )
    }

    @MainActor
    func confirmTransferScene(
        wallet: Wallet,
        data: GemTransferData,
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction,
    ) -> ConfirmTransferSceneViewModel {
        ConfirmTransferSceneViewModel(
            request: ConfirmTransferRequest(
                data: data,
                simulation: simulation,
                delegate: confirmTransferDelegate,
            ),
            wallet: wallet,
            confirmation: confirmTransferService().confirmation(wallet: wallet.toGem(), transfer: data, simulation: simulation),
            onComplete: { [toastPresenter] result in
                Task { toastPresenter.present(.transfer(result)) }
                onComplete?()
            },
        )
    }

    private func confirmTransferService() -> GemConfirmTransferService {
        GemConfirmTransferService(
            confirm: confirmService,
            explorer: explorerService,
            names: nameService,
            signer: KeystoreTransactionSigner(keystore: keystore),
            password: GemstoneKeystorePassword(keystore: keystore),
            recentActivity: recentAssetsService,
            preferences: preferencesService,
            payment: paymentService,
        )
    }

    @MainActor
    func receiveScene(assetData: AssetData, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetData: assetData, wallet: wallet, service: receiveService())
    }

    @MainActor
    func receiveScene(assetAddress: AssetAddress, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetAddress: assetAddress, wallet: wallet, service: receiveService())
    }

    private func receiveService() -> GemReceiveService {
        GemReceiveService(balances: balanceService, assets: assetsService, recentActivity: recentAssetsService)
    }

    @MainActor
    func recipientScene(
        wallet: Wallet,
        asset: Asset,
        type: GemRecipientType,
        recipient: GemPaymentRecipient? = .none,
        onNavigate: TransferRouteAction,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: GemRecipientService(payments: paymentService, session: walletSessionService),
            nameService: nameService,
            type: type,
            recipient: recipient,
            onNavigate: onNavigate,
        )
    }

    @MainActor
    func amountScene(
        input: AmountInput,
        wallet: Wallet,
        onTransferAction: TransferDataAction,
    ) -> AmountSceneViewModel {
        AmountSceneViewModel(
            input: input,
            wallet: wallet,
            service: amountService,
            stakeService: stakeService,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    func fiatScene(
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            service: GemFiatQuoteService(fiat: fiatService, balances: balanceService, session: walletSessionService, recentActivity: recentAssetsService),
            assetAddress: assetAddress,
            wallet: wallet,
            type: type,
            amount: amount,
        )
    }

    @MainActor
    func swapScene(
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
    func stakeScene(
        wallet: Wallet,
        chain: Chain,
        onNavigate: StakeRouteAction,
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: StakeChain(rawValue: chain.rawValue)!, // Expected Only StakeChain accepted.
            service: stakeService,
            onNavigate: onNavigate,
        )
    }

    @MainActor
    func earnScene(
        wallet: Wallet,
        asset: Asset,
        onNavigate: StakeRouteAction,
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: stakeService,
            onNavigate: onNavigate,
        )
    }

    @MainActor
    func delegationScene(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        validators: [DelegationValidator],
        onNavigate: StakeRouteAction,
        onSelectAddress: @escaping @MainActor @Sendable (ChainAddress) -> Void,
    ) -> DelegationSceneViewModel {
        DelegationSceneViewModel(
            wallet: wallet,
            delegation: delegation,
            asset: asset,
            service: stakeService,
            validators: validators,
            onNavigate: onNavigate,
            onSelectAddress: onSelectAddress,
        )
    }

    @MainActor
    func signMessageScene(
        request: GemWalletConnectMessageRequest,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            service: signMessageService,
            request: request,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
