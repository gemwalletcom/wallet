// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import QRScanner
import Style
import SwiftUI

public struct AddAssetScene: View {
    @State private var model: AddAssetSceneViewModel
    @State private var isPresentingUrl: URL?

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case address
    }

    private let onComplete: VoidAction

    public init(model: AddAssetSceneViewModel, onComplete: VoidAction) {
        _model = State(initialValue: model)
        self.onComplete = onComplete
    }

    public var body: some View {
        addTokenList
            .safeAreaButton {
                StateButton(
                    text: model.actionButtonTitle,
                    type: .primary(model.buttonState),
                    action: onSelectImportToken,
                )
            }
            .toolbarInfoButton(url: model.customTokenUrl)
            .onAppear {
                focusedField = .address
            }
            .onChange(of: model.input.address) {
                model.onChangeAddress()
            }
            .debouncedTask(id: model.loadTrigger) {
                await model.load()
            }
            .listSectionSpacing(.compact)
            .navigationTitle(model.title)
            .alertSheet($model.isPresentingAlertMessage)
            .navigationDestination(for: Scenes.NetworksSelector.self) { _ in
                NetworkSelectorScene(
                    model: model.networksModel,
                    onFinishSelection: onFinishChainSelection(chains:),
                )
            }
            .sheet(isPresented: $model.isPresentingScanner) {
                ScanQRCodeNavigationStack(scanType: .tokenContract, action: onHandleScan(_:))
            }
            .safariSheet(url: $isPresentingUrl)
    }
}

// MARK: - UI Components

extension AddAssetScene {
    private var addTokenList: some View {
        List {
            if let chain = model.input.chain {
                Section(model.networkTitle) {
                    if model.input.hasManyChains {
                        NavigationLink(value: Scenes.NetworksSelector()) {
                            ChainView(model: ChainViewModel(chain: chain))
                        }
                    } else {
                        ChainView(model: ChainViewModel(chain: chain))
                    }
                }
            }
            Section {
                FloatTextField(model.addressTitleField, text: model.addressBinding) {
                    HStack(spacing: .small) {
                        ListButton(image: model.pasteImage, action: onSelectPaste)
                        ListButton(image: model.qrImage, action: onSelectScan)
                    }
                }
                .focused($focusedField, equals: .address)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .submitLabel(.search)
                .onSubmit(model.onSubmitAddress)
            }

            if model.isLoading {
                ListItemLoadingView()
                    .id(UUID())
            }
            ForEach(model.sections.listSections) { section in
                Section {
                    ForEach(section.values) { GemListRowView(row: $0.row) }
                }
            }
            if model.showsVerificationWarning {
                Section {
                    ListItemView(model: model.warningListItem {
                        isPresentingUrl = model.tokenVerificationUrl
                    })
                }
            }
        }
    }
}

// MARK: - Actions

extension AddAssetScene {
    private func onFinishChainSelection(chains: [Chain]) {
        model.input.chain = chains.first
        model.input.address = nil
    }

    private func onSelectImportToken() {
        model.onSelectImportToken(onComplete: onComplete)
    }

    private func onSelectScan() {
        model.isPresentingScanner = true
    }

    private func onSelectPaste() {
        guard let address = UIPasteboard.general.string else { return }
        model.setInput(address)
        focusedField = nil
    }

    private func onHandleScan(_ result: String) {
        model.setInput(result)
        focusedField = nil
    }
}
