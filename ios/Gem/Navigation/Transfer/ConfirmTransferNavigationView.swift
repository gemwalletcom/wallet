// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import FiatConnect
import GemstonePrimitives
import InfoSheet
import Perpetuals
import Primitives
import PrimitivesComponents
import Swap
import SwiftUI
import Transfer

struct ConfirmTransferNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State var model: ConfirmTransferSceneViewModel

    var body: some View {
        ConfirmTransferScene(model: model)
            .sheet(item: $model.isPresentingSheet) {
                switch $0 {
                case let .info(type):
                    InfoSheetScene(type: type)
                case let .url(url):
                    SFSafariView(url: url)
                case .networkFeeSelector:
                    NetworkFeeSheet(model: model.feeModel)
                case let .paymentAsset(type):
                    SelectAssetSceneNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: model.assetAcquisitionWallet,
                            selectType: type,
                            selectAssetAction: model.selectPaymentAsset,
                        ),
                    )
                case let .paymentVerification(url):
                    PaymentVerificationScene(model: PaymentVerificationSceneViewModel(url: url, onComplete: model.onPaymentVerified))
                case .payloadDetails:
                    NavigationStack {
                        SimulationPayloadDetailsScene(
                            primaryModels: model.fieldModels(for: model.payloadModel.primaryFields),
                            secondaryModels: model.fieldModels(for: model.payloadModel.secondaryFields),
                        )
                    }
                    .sheetPresentation([.large])
                case let .fiatConnect(assetAddress, wallet, amount):
                    NavigationStack {
                        FiatConnectNavigationView(
                            model: viewModelFactory.fiatScene(assetAddress: assetAddress, wallet: wallet, amount: amount),
                        )
                        .navigationBarTitleDisplayMode(.inline)
                        .toolbarDismissItem(type: .close, placement: .topBarLeading)
                    }
                case let .getAsset(asset, buyAmount):
                    GetAssetNavigationStack(
                        asset: asset,
                        buyAmount: buyAmount,
                        model: model,
                        viewModelFactory: viewModelFactory,
                    )
                case let .selectedAsset(input, wallet):
                    SelectedAssetNavigationStack(
                        input: input,
                        wallet: wallet,
                        onComplete: { model.isPresentingSheet = nil },
                    )
                case .swapDetails:
                    if case let .swapDetails(model) = model.detailsViewModel.itemModel {
                        NavigationStack {
                            SwapDetailsView(model: Bindable(model))
                        }
                        .sheetPresentation(.forCurrentDeviceSize(expandable: true))
                    }
                case let .perpetualDetails(model):
                    NavigationStack {
                        PerpetualDetailsView(model: model)
                    }
                    .sheetPresentation(.forCurrentDeviceSize(expandable: true))
                case let .addressDetails(chainAddress):
                    AddressDetailsNavigationStack(model: viewModelFactory.addressDetailsScene(chainAddress: chainAddress))
                }
            }
    }
}

private struct GetAssetNavigationStack: View {
    private static let optionsDetent = PresentationDetent.height(360)

    let asset: Asset
    let buyAmount: Int?
    let model: ConfirmTransferSceneViewModel
    let viewModelFactory: ViewModelFactory

    @State private var selectedAction: GetAssetAction?
    @State private var actionNavigationPath = NavigationPath()

    var body: some View {
        NavigationStack {
            GetAssetScene(
                asset: asset,
                onSelect: {
                    actionNavigationPath = NavigationPath()
                    selectedAction = $0
                },
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .sheet(item: $selectedAction) { action in
                NavigationStack(path: $actionNavigationPath) {
                    destination(for: action)
                        .toolbarDismissItem(type: .close, placement: .topBarLeading)
                        .navigationBarTitleDisplayMode(.inline)
                        .navigationDestination(for: ConfirmTransferInput.self) { input in
                            ConfirmTransferNavigationView(
                                model: viewModelFactory.confirmTransferScene(
                                    wallet: model.assetAcquisitionWallet,
                                    data: input.data,
                                    onComplete: { model.isPresentingSheet = nil },
                                ),
                            )
                        }
                }
                .sheetPresentation([.large])
            }
        }
        .sheetPresentation([Self.optionsDetent])
    }

    @ViewBuilder
    private func destination(for type: GetAssetAction) -> some View {
        switch type {
        case .buy:
            FiatConnectNavigationView(
                model: viewModelFactory.fiatScene(
                    assetAddress: model.assetAddress(asset),
                    wallet: model.assetAcquisitionWallet,
                    amount: buyAmount,
                ),
            )
        case .swap:
            SwapNavigationView(
                model: viewModelFactory.swapScene(
                    input: SwapInput(
                        wallet: model.assetAcquisitionWallet,
                        pairSelector: model.acquireSwapPair(to: asset).map(),
                    ),
                    onSwap: { actionNavigationPath.append(ConfirmTransferInput(data: $0)) },
                ),
            )
        case .receive:
            ReceiveScene(model: viewModelFactory.receiveScene(assetAddress: model.assetAddress(asset), wallet: model.assetAcquisitionWallet))
        }
    }
}
