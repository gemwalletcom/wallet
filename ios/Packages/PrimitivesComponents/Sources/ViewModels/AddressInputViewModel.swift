// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemNameRecordState
import protocol Gemstone.GemNameServiceProtocol
import struct Gemstone.GemRecipient
import struct Gemstone.GemRecipientValidation
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
public final class AddressInputViewModel {
    let placeholder: String
    public let nameRecordViewModel: NameRecordViewModel
    private let nameService: any GemNameServiceProtocol

    public var chain: Chain {
        didSet { onChangeChain() }
    }

    var inputModel: InputValidationViewModel

    public init(
        chain: Chain,
        nameService: any GemNameServiceProtocol,
        placeholder: String,
    ) {
        self.chain = chain
        self.placeholder = placeholder
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
        self.nameService = nameService
        inputModel = InputValidationViewModel()
    }

    public var text: String {
        get { inputModel.text }
        set { inputModel.text = newValue }
    }

    public var nameResolveState: GemNameRecordState {
        nameRecordViewModel.state
    }

    public var isValid: Bool {
        validation.isValid
    }

    public var resolvedAddress: String {
        validation.address
    }

    private var validation: GemRecipientValidation {
        nameService.validateRecipient(chain: chain.rawValue, input: text, state: nameResolveState)
    }

    public func update(text: String) {
        self.text = text
        update(error: nil)
    }

    public func update(error: (any Error)?) {
        inputModel.update(error: error)
    }

    @discardableResult
    public func validate() -> Bool {
        guard text.isNotEmpty else {
            update(error: nil)
            return true
        }
        let validation = self.validation
        update(error: validation.error)
        return validation.isValid
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

    func onNameResolveStateChange(_: GemNameRecordState, newState: GemNameRecordState) {
        if newState.record() != nil {
            update(error: nil)
        }
    }
}

// MARK: - Private

extension AddressInputViewModel {
    private func onChangeChain() {
        nameRecordViewModel.reset()
        update(error: nil)
        nameRecordViewModel.getNameRecord(name: text, chain: chain)

        if nameRecordViewModel.state == .none, text.isNotEmpty {
            validate()
        }
    }
}
