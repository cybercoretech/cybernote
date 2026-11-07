const root = document.documentElement;
const quickDialog = document.getElementById('quickstart-dialog');
const toast = document.getElementById('toast');
// Themes: the universal Cybercore chooser (kit/chooser.js) in the header.
document.getElementById('quickstart-open').addEventListener('click', () => quickDialog.showModal());
document.querySelectorAll('dialog').forEach(dialog => dialog.addEventListener('click', event => {
  const box = dialog.getBoundingClientRect();
  if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close();
}));
