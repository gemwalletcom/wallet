// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct SuffixTextField<Field: Hashable>: View {
    private let placeholder: String
    private let suffix: String
    private let sanitizer: ((String) -> String)?
    private let alignment: TextAlignment
    @Binding private var text: String
    private let field: Field
    private var focusedField: FocusState<Field?>.Binding

    public init(
        placeholder: String = "",
        suffix: String,
        sanitizer: ((String) -> String)? = nil,
        alignment: TextAlignment = .trailing,
        text: Binding<String>,
        field: Field,
        focusedField: FocusState<Field?>.Binding,
    ) {
        self.placeholder = placeholder
        self.suffix = suffix
        self.sanitizer = sanitizer
        self.alignment = alignment
        _text = text
        self.field = field
        self.focusedField = focusedField
    }

    public var body: some View {
        HStack(spacing: .zero) {
            TextField(placeholder, text: $text)
                .keyboardType(.decimalPad)
                .multilineTextAlignment(alignment)
                .focused(focusedField, equals: field)
            Spacer()
                .frame(width: .tiny)
            Text(suffix)
                .foregroundStyle(Colors.gray)
        }
        .onChange(of: text) { _, newValue in
            guard let sanitizer else { return }
            let sanitized = sanitizer(newValue)
            if sanitized != newValue {
                text = sanitized
            }
        }
    }
}
