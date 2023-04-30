# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import merlon

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'Merlon'
copyright = """
2023, Alex Bates.

The author is not affiliated with Nintendo Co., Ltd. in any way.

The PAPER MARIO trademark owned by Nintendo Co., Ltd. is used in this modding tool under the fair use doctrine, solely
for the purpose of enabling users to modify the game in a transformative manner.

Mods packaged into a distributable with this application are encrypted using the original game ROM, and cannot be used
without the original game ROM. No guarantees are made about the legal status of using this application to create mods
"""
author = 'Alex Bates'
version = merlon.version()



# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.coverage',
    'myst_parser',
    'sphinx_copybutton'
]

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']

copybutton_prompt_text = "$ "
copybutton_exclude = '.linenos, .gp, .go'

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "sphinx_rtd_theme"
html_static_path = ['_static']
html_logo = '../assets/logo/logotype-docs.png'
html_favicon = '../assets/logo/merlon.ico'
html_theme_options = {
    'analytics_id': 'G-LCC88CM7GF',
    'analytics_anonymize_ip': True,
    'logo_only': True,
    'display_version': True,
    'prev_next_buttons_location': 'bottom',
    'style_external_links': False,
    'vcs_pageview_mode': '',
    'style_nav_header_background': '#534295',
    # Toc options
    'collapse_navigation': False,
    'sticky_navigation': True,
    'navigation_depth': 4,
    'includehidden': True,
    'titles_only': False
}



# --- Options for autodoc ---------------------------------------------------
autosummary_generate = True
autoclass_content = 'both'
