const root = document.documentElement;
const themeDialog = document.getElementById('theme-dialog');
const quickDialog = document.getElementById('quickstart-dialog');
const toast = document.getElementById('toast');
const storedTheme = localStorage.getItem('cybernote-theme');
if (storedTheme && ['cyan', 'plasma', 'ember', 'bio'].includes(storedTheme)) root.dataset.theme = storedTheme;
document.getElementById('theme-open').addEventListener('click', () => themeDialog.showModal());
document.getElementById('quickstart-open').addEventListener('click', () => quickDialog.showModal());
document.querySelectorAll('[data-theme-choice]').forEach(button => button.addEventListener('click', () => {
  root.dataset.theme = button.dataset.themeChoice;
  localStorage.setItem('cybernote-theme', button.dataset.themeChoice);
  themeDialog.close();
  toast.textContent = `✦ ${button.querySelector('b').textContent} selected`;
  toast.classList.add('show');
  window.setTimeout(() => toast.classList.remove('show'), 2200);
}));
document.querySelectorAll('dialog').forEach(dialog => dialog.addEventListener('click', event => {
  const box = dialog.getBoundingClientRect();
  if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close();
}));
