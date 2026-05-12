// Configured highlight.js core: registers only a curated set of languages
// to keep the bundle small while still covering the languages that show up
// most often in personal notes / reports.

import hljs from 'highlight.js/lib/core'

import bash from 'highlight.js/lib/languages/bash'
import c from 'highlight.js/lib/languages/c'
import cpp from 'highlight.js/lib/languages/cpp'
import csharp from 'highlight.js/lib/languages/csharp'
import css from 'highlight.js/lib/languages/css'
import diff from 'highlight.js/lib/languages/diff'
import dockerfile from 'highlight.js/lib/languages/dockerfile'
import go from 'highlight.js/lib/languages/go'
import ini from 'highlight.js/lib/languages/ini'
import java from 'highlight.js/lib/languages/java'
import javascript from 'highlight.js/lib/languages/javascript'
import json from 'highlight.js/lib/languages/json'
import kotlin from 'highlight.js/lib/languages/kotlin'
import markdown from 'highlight.js/lib/languages/markdown'
import php from 'highlight.js/lib/languages/php'
import plaintext from 'highlight.js/lib/languages/plaintext'
import powershell from 'highlight.js/lib/languages/powershell'
import python from 'highlight.js/lib/languages/python'
import ruby from 'highlight.js/lib/languages/ruby'
import rust from 'highlight.js/lib/languages/rust'
import scss from 'highlight.js/lib/languages/scss'
import shell from 'highlight.js/lib/languages/shell'
import sql from 'highlight.js/lib/languages/sql'
import swift from 'highlight.js/lib/languages/swift'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'

hljs.registerLanguage('bash',       bash)
hljs.registerLanguage('c',          c)
hljs.registerLanguage('cpp',        cpp)
hljs.registerLanguage('csharp',     csharp)
hljs.registerLanguage('css',        css)
hljs.registerLanguage('diff',       diff)
hljs.registerLanguage('dockerfile', dockerfile)
hljs.registerLanguage('go',         go)
hljs.registerLanguage('ini',        ini)
hljs.registerLanguage('java',       java)
hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('json',       json)
hljs.registerLanguage('kotlin',     kotlin)
hljs.registerLanguage('markdown',   markdown)
hljs.registerLanguage('php',        php)
hljs.registerLanguage('plaintext',  plaintext)
hljs.registerLanguage('powershell', powershell)
hljs.registerLanguage('python',     python)
hljs.registerLanguage('ruby',       ruby)
hljs.registerLanguage('rust',       rust)
hljs.registerLanguage('scss',       scss)
hljs.registerLanguage('shell',      shell)
hljs.registerLanguage('sql',        sql)
hljs.registerLanguage('swift',      swift)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('xml',        xml)
hljs.registerLanguage('yaml',       yaml)

// Common aliases — markdown-it passes the raw fence info string, so we
// want `js`, `ts`, `sh`, `html`, `vue` etc. to resolve to a registered
// language rather than falling through to plaintext.
hljs.registerAliases(['js', 'jsx', 'mjs', 'cjs'],          { languageName: 'javascript' })
hljs.registerAliases(['ts', 'tsx'],                        { languageName: 'typescript' })
hljs.registerAliases(['sh', 'zsh'],                        { languageName: 'bash' })
hljs.registerAliases(['ps', 'ps1', 'pwsh'],                { languageName: 'powershell' })
hljs.registerAliases(['html', 'htm', 'svg', 'vue', 'xhtml'], { languageName: 'xml' })
hljs.registerAliases(['py'],                               { languageName: 'python' })
hljs.registerAliases(['rs'],                               { languageName: 'rust' })
hljs.registerAliases(['rb'],                               { languageName: 'ruby' })
hljs.registerAliases(['kt', 'kts'],                        { languageName: 'kotlin' })
hljs.registerAliases(['cs'],                               { languageName: 'csharp' })
hljs.registerAliases(['c++', 'cxx', 'hpp', 'hxx'],         { languageName: 'cpp' })
hljs.registerAliases(['yml'],                              { languageName: 'yaml' })
hljs.registerAliases(['toml', 'conf'],                     { languageName: 'ini' })
hljs.registerAliases(['md', 'mkd'],                        { languageName: 'markdown' })
hljs.registerAliases(['docker'],                           { languageName: 'dockerfile' })
hljs.registerAliases(['text', 'txt'],                      { languageName: 'plaintext' })

export default hljs
