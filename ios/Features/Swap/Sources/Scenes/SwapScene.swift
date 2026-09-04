// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapScene: View {
    @FocusState private var focusedField: Bool
    @State private var isPresentingSlippage = false

    @State private var model: SwapSceneViewModel

    public init(model: SwapSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            swapFromSectionView
            swapToSectionView
            if model.shouldShowAdditionalInfo {
                additionalInfoSectionView
            }

            if let error = model.swapState.error {
                Section {
                    ListItemErrorView(
                        errorTitle: model.errorTitle,
                        error: error.asAnyError(asset: model.fromAsset?.asset),
                        infoAction: model.errorInfoAction,
                    )
                }
            }
        }
        .listSectionSpacing(.compact)
        .safeAreaView {
            bottomActionView
                .confirmationDialog(
                    model.swapDetailsViewModel?.highImpactWarningTitle ?? "",
                    presenting: $model.isPresentingPriceImpactConfirmation,
                    sensoryFeedback: .warning,
                    actions: { _ in
                        Button(
                            model.buttonViewModel.title,
                            role: .destructive,
                            action: model.onSelectSwapConfirmation,
                        )
                    },
                    message: {
                        Text(model.isPresentingPriceImpactConfirmation ?? "")
                    },
                )
        }
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                IndicatorButton(
                    systemImage: SystemImage.settings,
                    showsIndicator: model.showsSlippageIndicator,
                    action: { isPresentingSlippage = true },
                )
            }
        }
        .sheet(isPresented: $isPresentingSlippage) {
            if let slippageModel = model.swapSlippageViewModel {
                SwapSlippageScene(model: slippageModel)
                    .presentationDetents([.medium])
                    .presentationBackground(Colors.grayBackground)
            }
        }
        .onChangeBindQuery(model.fromAssetQuery, action: model.onChangeFromAsset)
        .onChangeBindQuery(model.toAssetQuery, action: model.onChangeToAsset)
        .debouncedTask(id: model.loadTrigger, interval: model.quoteDebounce) {
            await model.load()
        }
        .debounce(
            value: model.assetIds,
            initial: true,
            interval: .none,
            action: model.onAssetIdsChange,
        )
        .onChange(of: model.amountInputModel.text, model.onChangeFromValue)
        .onChange(of: model.pairSelectorModel, model.onChangePair)
        .onChange(of: model.pairSelectorModel.fromAssetId) { _, newValue in
            if newValue == nil {
                focusedField = false
            }
        }
        .onChange(of: model.selectedSwapQuote, model.onChangeSwapQuote)
        .onTimer(every: model.quoteRefreshInterval, id: model.loadTrigger) {
            await model.load()
        }
        .onAppear {
            focusedField = true
        }
        .task {
            await model.suggestPair()
        }
    }
}

// MARK: - UI Components

extension SwapScene {
    private var swapFromSectionView: some View {
        Section {
            SwapTokenView(
                model: model.swapTokenModel(type: .pay),
                text: $model.amountInputModel.text,
                onBalanceAction: model.onSelectFromMaxBalance,
                onSelectAssetAction: model.onSelectAssetPay,
            )
            .buttonStyle(.borderless)
            .focused($focusedField)
        } header: {
            Text(model.swapFromTitle)
                .listRowInsets(.horizontalMediumInsets)
        }
    }

    private var swapToSectionView: some View {
        Section {
            SwapTokenView(
                model: model.swapTokenModel(type: .receive(chains: [], assetIds: [])),
                text: $model.toValue,
                showLoading: model.isReceiveFieldLoading,
                onBalanceAction: {},
                onSelectAssetAction: model.onSelectAssetReceive,
            )
            .buttonStyle(.borderless)
        } header: {
            ZStack {
                Text(model.swapToTitle)
                    .frame(maxWidth: .infinity, alignment: .leading)
                SwapChangeView(
                    fromId: $model.pairSelectorModel.fromAssetId,
                    toId: $model.pairSelectorModel.toAssetId,
                )
                .padding(.vertical, .small)
                .offset(y: -.tiny)
                .disabled(model.isTransferDataLoading)
                .textCase(nil)
            }
            .listRowInsets(.horizontalMediumInsets)
        }
    }

    private var additionalInfoSectionView: some View {
        Section {
            if let swapDetailsViewModel = model.swapDetailsViewModel {
                NavigationCustomLink(
                    with: SwapDetailsListView(model: swapDetailsViewModel),
                    action: model.onSelectSwapDetails,
                )
            }
        }
    }

    private var swapButton: StateButton {
        StateButton(
            text: model.buttonViewModel.title,
            type: model.buttonViewModel.type,
            image: model.buttonViewModel.icon,
            infoTitle: model.buttonViewModel.infoText,
            action: onSelectActionButton,
        )
    }

    private var bottomActionView: some View {
        InputAccessoryView(
            isEditing: focusedField && !model.buttonViewModel.isVisible,
            suggestions: SwapSceneViewModel.inputPercentSuggestions,
            onSelect: {
                focusedField = false
                model.onSelectPercent($0.value)
            },
            onDone: { focusedField = false },
            button: swapButton,
        )
    }
}

// MARK: - Actions

extension SwapScene {
    private func onSelectActionButton() {
        focusedField = false
        model.buttonViewModel.action()
    }
}
