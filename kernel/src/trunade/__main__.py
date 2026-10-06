from ipykernel.kernelapp import IPKernelApp

from trunade.kernel import TrunadeKernel

IPKernelApp.launch_instance(kernel_class=TrunadeKernel)
