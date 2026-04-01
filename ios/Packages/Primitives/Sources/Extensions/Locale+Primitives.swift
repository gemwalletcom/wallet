// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Locale {
    static let US = Locale(identifier: "en_US")
    static let UK = Locale(identifier: "en_UK")
    static let UA = Locale(identifier: "ru_UA")
    static let IT = Locale(identifier: "it_IT")
    static let PT_BR = Locale(identifier: "pt-BR")
    static let DA_DK = Locale(identifier: "DA_DK")
    static let PT_PT = Locale(identifier: "pt-PT")
    static let FR = Locale(identifier: "fr-CA")
    static let EN_CH = Locale(identifier: "en_CH")
    static let DE_CH = Locale(identifier: "de_CH")
    static let RU_UA = Locale(identifier: "ru_UA")
    static let ZH_Simplifier = Locale(identifier: "zh-Hans")
    static let ZH_Singapore = Locale(identifier: "zh_SG")
    static let ZH_Traditional = Locale(identifier: "zh-Hant")
    static let AR_SA = Locale(identifier: "ar_SA")

    func usageLanguageIdentifier() -> String {
        guard let languageCode = language.languageCode else {
            return Locale.US.language.minimalIdentifier
        }
        if let script = language.script, languageCode.identifier == Locale.ZH_Simplifier.language.languageCode?.identifier {
            return "\(languageCode.identifier)-\(script.identifier)"
        }
        return languageCode.identifier
    }

    internal func usageAppstoreLanguageIdentifier() -> String {
        guard let languageCode = language.languageCode else {
            return Locale.US.language.minimalIdentifier
        }
        if let script = language.script, languageCode.identifier == Locale.ZH_Simplifier.language.languageCode?.identifier {
            return "\(languageCode.identifier)-\(script.identifier)"
        }
        if let region = language.region {
            return "\(languageCode.identifier)-\(region.identifier)"
        }
        return languageCode.identifier
    }

    // https://github.com/fastlane/fastlane/blob/e1ab951b63aeb7a669b415708ffbe0e1346c0f59/fastlane_core/lib/fastlane_core/languages.rb#L14
    func appstoreLanguageIdentifier() -> String {
        guard let languageCode = language.languageCode else {
            return Locale.US.language.minimalIdentifier
        }
        return switch languageCode {
        case .german: "de-DE"
        case .norwegian,
             .norwegianBokmål,
             .norwegianNynorsk: "no"
        case .italian,
             .vietnamese,
             .czech,
             .japanese,
             .korean,
             .russian,
             .turkish,
             .ukrainian,
             .vietnamese,
             .danish,
             .greek,
             .finnish,
             .hebrew,
             .hindi,
             .croatian,
             .indonesian,
             .malay,
             .polish,
             .romanian,
             .slovak,
             .swedish,
             .catalan,
             .thai,
             .hungarian: languageCode.identifier
        case .chinese: usageAppstoreLanguageIdentifier()
        default: usageAppstoreLanguageIdentifier()
        }
    }
}
