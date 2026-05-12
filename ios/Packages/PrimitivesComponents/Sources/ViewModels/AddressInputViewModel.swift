// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI
import Validators

@Observable
@MainActor
public final class AddressInputViewModel {
    let placeholder: String
    public let nameRecordViewModel: NameRecordViewModel

    public var chain: Chain {
        didSet { onChangeChain() }
    }

    var inputModel: InputValidationViewModel

    public init(
        chain: Chain,
        nameService: any NameServiceable,
        placeholder: String,
        validators: [any TextValidator] = [],
    ) {
        self.chain = chain
        self.placeholder = placeholder
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
        inputModel = InputValidationViewModel(
            mode: .manual,
            validators: validators,
        )
    }

    public var text: String {
        get { inputModel.text }
        set { inputModel.text = (try? chain.checksumAddress(newValue)) ?? newValue }
    }

    public var nameResolveState: NameRecordState {
        nameRecordViewModel.state
    }

    public var isValid: Bool {
        switch nameResolveState {
        case .none: inputModel.isValid && inputModel.text.isNotEmpty
        case .loading, .error: false
        case .complete: true
        }
    }

    public var address: String {
        let raw = nameResolveState.result?.address ?? inputModel.text.trim()
        return (try? chain.checksumAddress(raw)) ?? raw
    }

    @discardableResult
    public func update() -> Bool {
        inputModel.update()
    }

    public func update(text: String) {
        inputModel.update(text: (try? chain.checksumAddress(text)) ?? text)
    }

    public func update(error: (any Error)?) {
        inputModel.update(error: error)
    }

    @discardableResult
    public func validate() -> Bool {
        if nameRecordViewModel.canResolveName(name: text) {
            isValid
        } else {
            update()
        }
    }

    public func updateValidators(_ validators: [any TextValidator]) {
        inputModel.update(validators: validators)
    }
}

extension AddressInputViewModel {
    public var shouldShowInputActions: Bool {
        inputModel.text.isEmpty
    }

    func onSelectPaste() {
        guard let address = UIPasteboard.general.string else { return }
        update(text: address)
    }

    func onTextChange(_: String, newText: String) {
        nameRecordViewModel.resolve(name: newText, chain: chain)
    }

    func onNameResolveStateChange(_: NameRecordState, newState: NameRecordState) {
        if newState.result != nil {
            update(error: nil)
        }
    }
}

// MARK: - Private

extension AddressInputViewModel {
    private func onChangeChain() {
        nameRecordViewModel.reset()
        let currentText = text

        inputModel = InputValidationViewModel(
            mode: .manual,
            validators: [
                .required(requireName: placeholder),
                .address(Asset(chain)),
            ],
        )
        text = currentText

        if nameRecordViewModel.canResolveName(name: currentText) {
            nameRecordViewModel.resolve(name: currentText, chain: chain)
        } else if currentText.isNotEmpty {
            inputModel.update()
        }
    }
}
