// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Validators
import WalletCorePrimitives

@Observable
@MainActor
public final class ManageContactAddressViewModel {
    public enum Mode: Identifiable {
        case add
        case edit(ContactAddress)

        public var id: String {
            switch self {
            case .add: "add"
            case let .edit(address): address.id
            }
        }

        var contactAddress: ContactAddress? {
            switch self {
            case .add: nil
            case let .edit(address): address
            }
        }
    }

    private let mode: Mode
    private let contactId: String
    private let onComplete: (ContactAddress) -> Void

    var addressInputModel: AddressInputViewModel
    var memo: String = ""
    var isPresentingScanner = false

    public init(
        contactId: String,
        nameService: any NameServiceable,
        mode: Mode,
        onComplete: @escaping (ContactAddress) -> Void,
    ) {
        self.contactId = contactId
        self.mode = mode
        self.onComplete = onComplete
        title = Localized.Common.address

        let chain = mode.contactAddress?.chain ?? .bitcoin
        addressInputModel = AddressInputViewModel(
            chain: chain,
            nameService: nameService,
            placeholder: title,
            validators: [.required(requireName: title), .address(Asset(chain))],
        )

        if let address = mode.contactAddress {
            addressInputModel.text = address.address
            memo = address.memo ?? ""
        }
    }

    let title: String
    var buttonTitle: String { Localized.Transfer.confirm }
    var networkTitle: String { Localized.Transfer.network }
    var memoTitle: String { Localized.Transfer.memo }
    var chain: Chain { addressInputModel.chain }
    var showMemo: Bool { chain.isMemoSupported }

    var networkSelectorModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(Chain.allCases)),
            selectedItems: [chain],
            selectionType: .checkmark,
        )
    }

    var buttonState: ButtonState {
        addressInputModel.isValid ? .normal : .disabled
    }

    private var currentAddress: ContactAddress {
        ContactAddress.new(
            contactId: contactId,
            chain: chain,
            address: chain.checksumAddress(addressInputModel.resolvedAddress),
            memo: memo.isEmpty ? nil : memo,
        )
    }
}

// MARK: - Actions

extension ManageContactAddressViewModel {
    func onSelectChain(_ chain: Chain) {
        addressInputModel.chain = chain
        memo = ""
    }

    func onSelectScan() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        addressInputModel.update(text: result)
    }

    func complete() {
        guard addressInputModel.validate() else { return }
        onComplete(currentAddress)
    }
}
