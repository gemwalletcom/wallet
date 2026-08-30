// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemRecipientService
import struct Gemstone.GemRecipientValidation
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
    private let recipientService: GemRecipientService

    public var chain: Chain {
        didSet { onChangeChain() }
    }

    var inputModel: InputValidationViewModel

    public init(
        chain: Chain,
        nameService: any GemNameServiceProtocol,
        placeholder: String,
        validators: [any TextValidator] = [],
    ) {
        self.chain = chain
        self.placeholder = placeholder
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
        recipientService = nameService.recipients()
        inputModel = InputValidationViewModel(
            mode: .manual,
            validators: validators,
        )
    }

    public var text: String {
        get { inputModel.text }
        set { inputModel.text = newValue }
    }

    public var nameResolveState: NameRecordState {
        nameRecordViewModel.state
    }

    public var isValid: Bool {
        switch nameResolveState {
        case .none: inputModel.isValid && validation.isValid
        case .loading, .error: false
        case .complete: validation.isValid
        }
    }

    public var resolvedAddress: String {
        validation.address
    }

    public func recipient(memo: String?, references: [String] = []) throws -> Recipient {
        try Recipient(recipientService.recipient(
            chain: chain.rawValue,
            input: text,
            nameRecord: nameResolveState.result?.json(),
            memo: memo,
            references: references,
        ))
    }

    private var validation: GemRecipientValidation {
        recipientService.validate(chain: chain.rawValue, input: text, nameRecord: nameResolveState.result?.json())
    }

    @discardableResult
    public func update() -> Bool {
        inputModel.update()
    }

    public func update(text: String) {
        inputModel.update(text: text)
    }

    public func update(error: (any Error)?) {
        inputModel.update(error: error)
    }

    @discardableResult
    public func validate() -> Bool {
        if nameRecordViewModel.isNameSupported(name: text) {
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
        nameRecordViewModel.getNameRecord(name: newText, chain: chain)
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

        if nameRecordViewModel.isNameSupported(name: currentText) {
            nameRecordViewModel.getNameRecord(name: currentText, chain: chain)
        } else if currentText.isNotEmpty {
            inputModel.update()
        }
    }
}
