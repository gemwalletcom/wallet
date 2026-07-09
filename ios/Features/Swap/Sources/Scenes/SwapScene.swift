// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapScene: View {
    @FocusState private var focusedField: Bool

    @State private var model: SwapSceneViewModel

    public init(model: SwapSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: .medium) {
                swapInputView

                if model.shouldShowAdditionalInfo {
                    additionalInfoSectionView
                }

                errorSectionView
            }
            .padding(.medium)
        }
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
        .background(Colors.grayBackground.ignoresSafeArea())
        .navigationTitle(model.title)
        .onChangeBindQuery(model.fromAssetQuery, action: model.onChangeFromAsset)
        .onChangeBindQuery(model.toAssetQuery, action: model.onChangeToAsset)
        .debouncedTask(id: model.fetchTrigger) {
            await model.fetch()
        }
        .debounce(
            value: model.assetIds,
            initial: true,
            interval: .none,
            action: model.onAssetIdsChange,
        )
        .onChange(of: model.amountInputModel.text, model.onChangeFromValue)
        .onChange(of: model.pairSelectorModel, model.onChangePair)
        .onChange(of: model.selectedSwapQuote, model.onChangeSwapQuote)
        .onTimer(every: 30, id: model.fetchTrigger) {
            await model.fetch()
        }
        .onAppear {
            focusedField = true
        }
    }
}

// MARK: - UI Components

extension SwapScene {
    private var swapInputView: some View {
        ZStack {
            VStack(spacing: .tiny) {
                swapFromView
                swapToView
            }

            SwapChangeView(
                fromId: $model.pairSelectorModel.fromAssetId,
                toId: $model.pairSelectorModel.toAssetId,
            )
            .disabled(model.isTransferDataLoading)
        }
    }

    private var swapFromView: some View {
        SwapTokenView(
            model: model.swapTokenModel(type: .pay),
            text: $model.amountInputModel.text,
            onBalanceAction: model.onSelectFromMaxBalance,
            onSelectAssetAction: model.onSelectAssetPay,
        )
        .buttonStyle(.borderless)
        .focused($focusedField)
        .insetGroupedRow()
    }

    private var swapToView: some View {
        SwapTokenView(
            model: model.swapTokenModel(type: .receive(chains: [], assetIds: [])),
            text: $model.toValue,
            showLoading: model.isReceiveFieldLoading,
            onBalanceAction: {},
            onSelectAssetAction: model.onSelectAssetReceive,
        )
        .buttonStyle(.borderless)
        .insetGroupedRow()
    }

    @ViewBuilder
    private var additionalInfoSectionView: some View {
        if let swapDetailsViewModel = model.swapDetailsViewModel {
            Button(action: model.onSelectSwapDetails) {
                HStack(spacing: .small) {
                    SwapDetailsListView(model: swapDetailsViewModel)
                    disclosureChevron
                }
            }
            .tint(Colors.black)
            .insetGroupedRow()
        }
    }

    @ViewBuilder
    private var errorSectionView: some View {
        if let error = model.swapState.error {
            let errorView = ListItemErrorView(
                errorTitle: model.errorTitle,
                error: error.asAnyError(asset: model.fromAsset?.asset),
            )
            if let infoAction = model.errorInfoAction {
                Button(action: infoAction) {
                    HStack(spacing: .small) {
                        errorView
                        disclosureChevron
                    }
                }
                .tint(Colors.black)
                .insetGroupedRow()
            } else {
                errorView.insetGroupedRow()
            }
        }
    }

    private var disclosureChevron: some View {
        Images.System.chevronRight
            .font(.footnote.weight(.semibold))
            .foregroundStyle(Colors.grayLight)
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
