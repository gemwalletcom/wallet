// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import struct Stake.ValidatorView
import struct Stake.ValidatorViewModel
import Style
import SwiftUI

public struct AmountScene: View {
    @FocusState private var focusedField: Bool

    private var model: AmountSceneViewModel

    public init(model: AmountSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        @Bindable var model = model
        List {
            CurrencyInputValidationView(
                text: $model.amountInputModel.text,
                error: model.amountInputModel.error,
                config: model.inputConfig,
                infoAction: model.infoAction(for:),
            )
            .padding(.top, .medium)
            .listGroupRowStyle()
            .disabled(model.isInputDisabled)
            .focused($focusedField)

            if model.isBalanceViewEnabled {
                Section {
                    AssetBalanceView(
                        image: model.assetImage,
                        title: model.assetName,
                        balance: model.balanceText,
                        secondary: {
                            Button(model.maxTitle, action: onSelectMaxButton)
                                .buttonStyle(.listEmpty(paddingHorizontal: .medium, paddingVertical: .small))
                                .fixedSize()
                        },
                    )
                }
            }

            if let infoText = model.infoText {
                Section {
                    Button(action: model.onSelectReservedFeesInfo) {
                        HStack {
                            Images.System.info
                                .foregroundStyle(Colors.gray)
                                .frame(width: .list.image, height: .list.image)
                            Text(infoText)
                                .textStyle(.calloutSecondary)
                        }
                    }
                }
            }

            switch model.extras {
            case let .validator(row, canSelect):
                Section(model.validatorTitle) {
                    if canSelect {
                        NavigationLink(value: row.validator.toPrimitives()) {
                            ValidatorView(model: ValidatorViewModel(row: row))
                        }
                    } else {
                        ValidatorView(model: ValidatorViewModel(row: row))
                    }
                }
            case let .resources(options, selected):
                Section {
                    Picker("", selection: model.resourceBinding(selected: selected.toPrimitives())) {
                        ForEach(options.map { $0.toPrimitives() }) { resource in
                            Text(resource.title)
                                .tag(resource)
                        }
                    }
                    .pickerStyle(.segmented)
                    .frame(width: Sizing.picker.segmentedWidth)
                }
                .cleanListRow()
            case let .provider(row):
                Section(model.providerTitle) {
                    ValidatorView(model: ValidatorViewModel(row: row))
                }
            case let .perpetual(leverage, autoclose):
                if let leverage {
                    Section {
                        NavigationCustomLink(
                            with: ListItemView(model: model.leverageListItem(leverage)),
                            action: model.onSelectLeverage,
                        )
                    }
                }
                if let autoclose, let autocloseListItem = autoclose.listItemModel(onInfo: model.onInfo) {
                    Section {
                        NavigationCustomLink(
                            with: ListItemView(model: autocloseListItem),
                            action: model.onSelectAutoclose,
                        )
                    }
                }
            case .none:
                EmptyView()
            }
        }
        .safeAreaButton {
            StateButton(
                text: model.continueTitle,
                type: .primary(model.actionButtonState),
                action: onSelectNextButton,
            )
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if model.transferState.isLoading {
                    ProgressView()
                } else {
                    Button(model.continueTitle, action: onSelectNextButton)
                        .bold()
                        .disabled(!model.isNextEnabled)
                }
            }
        }
        .contentMargins([.top], .zero, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .frame(maxWidth: .infinity)
        .navigationTitle(model.title)
        .onChange(of: model.amountInputModel.text, model.onChangeAmountText)
        .taskOnce {
            model.prefillAmount()
            if model.input.focusesInput {
                focusedField = true
            }
        }
        .onDisappear {
            focusedField = false
        }
    }

    private func onSelectMaxButton() {
        focusedField = false
        model.onSelectMaxButton()
    }

    private func onSelectNextButton() {
        focusedField = false
        model.onSelectNextButton()
    }
}
