// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct AddressInputView: View {
    @Binding private var model: AddressInputViewModel

    private let onSelectScan: (@MainActor () -> Void)?
    private let onSelectPaste: (@MainActor () -> Void)?

    public init(
        model: Binding<AddressInputViewModel>,
        onSelectScan: (@MainActor () -> Void)? = nil,
        onSelectPaste: (@MainActor () -> Void)? = nil,
    ) {
        _model = model
        self.onSelectScan = onSelectScan
        self.onSelectPaste = onSelectPaste
    }

    public var body: some View {
        InputValidationField(
            model: $model.inputModel,
            placeholder: model.placeholder,
            allowClean: true,
            trailingView: {
                HStack(spacing: Spacing.medium) {
                    NameRecordView(model: model.nameRecordViewModel)
                    if model.shouldShowInputActions {
                        ListButton(image: Images.System.paste, action: onSelectPaste ?? model.onSelectPaste)
                        if let onSelectScan {
                            ListButton(image: Images.System.qrCodeViewfinder, action: onSelectScan)
                        }
                    }
                }
            },
        )
        .keyboardType(.alphabet)
        .textInputAutocapitalization(.never)
        .autocorrectionDisabled()
        .onChange(of: model.text, model.onTextChange)
        .onChange(of: model.nameResolveState, model.onNameResolveStateChange)
        .onSubmit { model.validate() }
    }
}
