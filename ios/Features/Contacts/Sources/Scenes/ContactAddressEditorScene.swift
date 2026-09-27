// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.chainRow
import enum Gemstone.GemContactAddressField
import Primitives
import PrimitivesComponents
import QRScanner
import Style
import SwiftUI

public struct ContactAddressEditorScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: ContactAddressEditorSceneViewModel

    @FocusState private var focusedField: Field?
    enum Field: Int, Hashable {
        case address
        case memo
    }

    public init(model: ContactAddressEditorSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.fields, id: \.self) { field in
                section(for: field)
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("", systemImage: SystemImage.checkmark, action: onComplete)
                    .disabled(model.buttonState == .disabled)
            }
        }
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            focusedField = .address
        }
        .sheet(isPresented: $model.isPresentingScanner) {
            QRScannerNavigationStack(scanType: .address, action: onScan)
        }
        .navigationDestination(for: Scenes.NetworksSelector.self) { _ in
            NetworkSelectorScene(
                model: model.networkSelectorModel,
                onFinishSelection: onFinishChainSelection(chains:),
            )
        }
    }
}

// MARK: - UI Components

extension ContactAddressEditorScene {
    @ViewBuilder
    private func section(for field: GemContactAddressField) -> some View {
        switch field {
        case .network: chainSection
        case .address: addressSection
        case .memo: memoSection
        }
    }

    private var chainSection: some View {
        Section(model.networkTitle) {
            NavigationLink(value: Scenes.NetworksSelector()) {
                ChainView(model: chainRow(chain: model.chain.rawValue))
            }
        }
    }

    private var addressSection: some View {
        Section {
            AddressInputView(
                model: $model.addressInputModel,
                onSelectScan: model.onSelectScan,
                onSelectPaste: model.onSelectPaste,
            )
            .focused($focusedField, equals: .address)
        }
    }

    private var memoSection: some View {
        Section {
            FloatTextField(
                model.memoTitle,
                text: $model.memo,
                allowClean: true,
            )
            .focused($focusedField, equals: .memo)
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
        }
    }
}

// MARK: - Actions

extension ContactAddressEditorScene {
    private func onFinishChainSelection(chains: [Chain]) {
        guard let chain = chains.first else { return }
        model.onSelectChain(chain)
    }

    private func onScan(_ result: String) {
        model.onScan(result)
        focusedField = nil
    }

    private func onComplete() {
        focusedField = nil
        model.complete()
        dismiss()
    }
}
