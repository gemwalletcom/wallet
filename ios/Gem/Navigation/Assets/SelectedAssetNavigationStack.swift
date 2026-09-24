// Copyright (c). Gem Wallet. All rights reserved.

import FiatConnect
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Stake
import Swap
import SwiftUI
import Transfer

struct SelectedAssetNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.navigationPresenter) private var presenter

    @State private var navigationPath = NavigationPath()
    @State private var isPresentingAddressDetails: ChainAddress?

    private let input: SelectedAssetInput
    private let wallet: Wallet
    private let onComplete: VoidAction

    init(
        input: SelectedAssetInput,
        wallet: Wallet,
        onComplete: VoidAction,
    ) {
        self.input = input
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            Group {
                switch input.type {
                case let .send(type):
                    RecipientNavigationView(
                        model: viewModelFactory.recipientScene(
                            wallet: wallet,
                            asset: input.asset,
                            type: type,
                            recipient: input.recipient,
                            onNavigate: navigate,
                        ),
                    )
                case .receive:
                    ReceiveScene(model: viewModelFactory.receiveScene(assetData: input.assetData, wallet: wallet))
                case let .buy(_, amount):
                    FiatConnectNavigationView(
                        model: viewModelFactory.fiatScene(
                            assetAddress: input.assetAddress,
                            wallet: wallet,
                            type: .buy,
                            amount: amount,
                        ),
                    )
                case let .sell(_, amount):
                    FiatConnectNavigationView(
                        model: viewModelFactory.fiatScene(
                            assetAddress: input.assetAddress,
                            wallet: wallet,
                            type: .sell,
                            amount: amount,
                        ),
                    )
                case let .swap(fromAssetId, toAssetId):
                    SwapNavigationView(
                        model: viewModelFactory.swapScene(
                            input: SwapInput(
                                wallet: wallet,
                                pairSelector: SwapPairSelectorViewModel(
                                    fromAssetId: fromAssetId,
                                    toAssetId: toAssetId,
                                ),
                            ),
                            onSwap: { navigate(to: .confirm($0)) },
                        ),
                    )
                case .stake:
                    StakeNavigationView(
                        model: viewModelFactory.stakeScene(
                            wallet: wallet,
                            chain: input.asset.id.chain,
                            onNavigate: navigate,
                        ),
                    )
                case .earn:
                    EarnNavigationView(
                        model: viewModelFactory.earnScene(
                            wallet: wallet,
                            asset: input.asset,
                            onNavigate: navigate,
                        ),
                    )
                }
            }
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: ConfirmTransferInput.self) { confirm in
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: wallet,
                        data: confirm.data,
                        onComplete: onComplete,
                    ),
                )
            }
            .navigationDestination(for: AmountInput.self) { amount in
                AmountNavigationView(
                    model: viewModelFactory.amountScene(
                        input: amount,
                        wallet: wallet,
                        onTransferAction: { navigate(to: .confirm($0)) },
                    ),
                )
            }
            .navigationDestination(for: DelegationInput.self) { input in
                DelegationScene(
                    model: viewModelFactory.delegationScene(
                        wallet: wallet,
                        delegation: input.delegation,
                        asset: input.delegation.base.assetId.chain.asset,
                        validators: input.validators,
                        onNavigate: navigate,
                        onSelectAddress: { isPresentingAddressDetails = $0 },
                    ),
                )
            }
        }
        .sheet(isPresented: Binding(get: { isPresentingAddressDetails != nil }, set: {
            if !$0 {
                isPresentingAddressDetails = nil
            }
        })) {
            if let isPresentingAddressDetails {
                AddressDetailsDestination(chainAddress: isPresentingAddressDetails)
            }
        }
    }
}

// MARK: - Actions

extension SelectedAssetNavigationStack {
    private func navigate(to route: TransferRoute) {
        switch route {
        case let .amount(input): navigationPath.append(input)
        case let .confirm(data): navigationPath.append(ConfirmTransferInput(data: data))
        }
    }

    private func navigate(to route: StakeRoute) {
        switch route {
        case let .delegation(input): navigationPath.append(input)
        case let .transfer(transfer): navigate(to: transfer)
        }
    }
}
