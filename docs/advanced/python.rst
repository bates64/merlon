Python
======

Installation
------------

Merlon is available as a package on PyPI.

.. important::

   Merlon requires Python 3.7 or newer.

You can install it with ``pip``:

.. code-block:: console

   $ pip install merlon

Usage
-----

Once installed, you can import the library in your Python code:

.. code-block:: python

   import merlon

   print("Merlon version:", merlon.version())
   print("Current package:", merlon.package.Package.current())

API Reference
-------------

.. automodule:: merlon
   :members:

.. automodule:: merlon.package
   :members:

.. automodule:: merlon.package.manifest
   :members:

.. automodule:: merlon.package.distribute
   :members:

.. automodule:: merlon.package.init
   :members:

.. automodule:: merlon.package.registry
   :members:

.. automodule:: merlon.emulator
   :members:

.. automodule:: merlon.rom
   :members:

.. toctree::
   :maxdepth: 2
   :caption: Contents:
