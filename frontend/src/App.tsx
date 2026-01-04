import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Layout } from '@/components/Layout';
import { Dashboard, Products, Attributes, Rules, Settings, Functionalities, Datatypes, Enumerations, RulesExplorer } from '@/pages';
import './index.css';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="products" element={<Products />} />
          <Route path="datatypes" element={<Datatypes />} />
          <Route path="enumerations" element={<Enumerations />} />
          <Route path="attributes" element={<Attributes />} />
          <Route path="functionalities" element={<Functionalities />} />
          <Route path="rules" element={<Rules />} />
          <Route path="explorer" element={<RulesExplorer />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
